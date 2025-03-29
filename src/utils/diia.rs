use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};


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

    // Make the GET request asynchronously
    let response = client
        .get(format!(
            "{}/api/v1/auth/acquirer/{}",
            state.config.diia.host,
            state.config.diia.acquirer_token
        ))
        .headers(headers)
        .send()
        .await?;

    // Get the response body as text
    let body = response.text().await?;

    let mut lock = state.diia_session_token.lock().await;
    *lock = body;

    Ok(())
}
