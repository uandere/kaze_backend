use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use serde::Deserialize;
use tracing::{error, info};
use anyhow::anyhow;


use crate::commands::server::ServerState;
use super::server_error::ServerError;

#[derive(Deserialize)]
pub struct SessionTokenResponse {
    token: String,
}


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
    let body: SessionTokenResponse = serde_json::from_str(&response.text().await?)?;
    info!("Successfully refreshed Diia token");

    // Store the raw response
    let mut lock = state.diia_session_token.lock().await;
    *lock = body.token;

    Ok(())
}