use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use tracing::{error, info};
use anyhow::anyhow;


use crate::commands::server::ServerState;
use super::server_error::ServerError;


/// This function refreshes the Diia session token.
pub async fn refresh_diia_session_token(state: ServerState) -> Result<(), ServerError> {
    let client = reqwest::Client::new();

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", state.config.diia.auth_acquirer_token))?,
    );

    // Log the exact request we're making for debugging
    let url = format!(
        "{}/api/v1/auth/acquirer/{}",
        state.config.diia.host,
        state.config.diia.acquirer_token
    );
    info!("Refreshing Diia token - URL: {}", url);
    info!("Using Authorization: Basic {}", state.config.diia.auth_acquirer_token);

    // Make the GET request asynchronously
    let response = client
        .get(&url)
        .headers(headers)
        .send()
        .await?;

    // Check the status code - important!
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        error!("Failed to refresh Diia token. Status: {}, Response: {}", status, error_text);
        return Err(anyhow!("Diia API returned error status {}: {}", status, error_text).into());
    }

    // Get the response body as text
    let body = response.text().await?;
    info!("Successfully got Diia token response");

    // If it's JSON, extract the token field
    if body.starts_with('{') {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(token) = parsed.get("token").and_then(|v| v.as_str()) {
                let mut lock = state.diia_session_token.lock().await;
                *lock = token.to_string();
                info!("Successfully extracted token from JSON response");
                return Ok(());
            }
        }
    }

    // Store the raw response
    let mut lock = state.diia_session_token.lock().await;
    *lock = body;
    info!("Stored raw token response");

    Ok(())
}