use crate::{commands::server::ServerState, utils::{diia::refresh_diia_session_token, server_error::ServerError}};
use anyhow::anyhow;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Payload {
    /// This is a backdoor for testing purposes
    pub _uid: String,
}

#[derive(Serialize)]
pub struct Response {
    deeplink: String,
}

#[derive(Serialize, Deserialize)]
pub struct DiiaSharingRequestId {
    pub uid: String,
    pub seed: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiiaSharingRequest {
    pub offer_id: String,
    pub return_link: String,
    pub request_id: String,
}

#[derive(Deserialize)]
pub struct DiiaSharingResponse {
    deeplink: String,
}

/// Generates an authorization link for Diia sharing.
pub async fn handler(
    State(state): State<ServerState>,
    Json(payload): Json<Payload>,
) -> Result<Json<Response>, ServerError> {
    let uid = payload._uid;
    info!("Point 0");

    // Check if the Diia session token is available
    let diia_token = state.diia_session_token.lock().await.clone();
    if diia_token.is_empty() {
        // Token is empty, try to refresh it first
        info!("Diia token is empty, attempting to refresh it");
        refresh_diia_session_token(state.clone()).await?;
        
        // Get the refreshed token
        let refreshed_token = state.diia_session_token.lock().await.clone();
        if refreshed_token.is_empty() {
            return Err(anyhow!("Failed to obtain Diia session token").into());
        }
    }

    let request_id = DiiaSharingRequestId {
        uid,
        seed: Uuid::new_v4(),
    };

    // Serialize request_id to string and handle errors properly
    let request_id_str = match serde_json::to_string(&request_id) {
        Ok(str) => str,
        Err(e) => return Err(anyhow!("Failed to serialize request ID: {}", e).into()),
    };

    // setting up the request
    let request = DiiaSharingRequest {
        offer_id: state.config.diia.offer_id.clone(),
        return_link: "https://mykaze.org".into(),
        request_id: request_id_str,
    };

    // setting up endpoint
    let base_url = format!("{}/api/v2/acquirers/branch", state.config.diia.host);
    let endpoint = format!(
        "{}/{}/offer-request/dynamic",
        base_url, state.config.diia.branch_id
    );

    info!("Point 1 - Endpoint: {}", endpoint);

    // Get token again in case it was refreshed
    let token = state.diia_session_token.lock().await.clone();
    info!("Using Diia token (first 10 chars): {}", if token.len() > 10 { &token[0..10] } else { &token });

    // sending the request with better error handling
    let client = reqwest::Client::new();
    let response = match client
        .post(&endpoint)
        .header("accept", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await {
            Ok(resp) => resp,
            Err(e) => {
                info!("Request failed: {:?}", e);
                return Err(anyhow!("Failed to send request to Diia API: {}", e).into());
            }
        };

    info!("Point 2 - Got response with status: {}", response.status());
    
    // getting a deeplink and returning it
    if response.status().is_success() {
        let body = response.text().await?;
        info!("Response body: {}", body);
        
        let api_response: DiiaSharingResponse = match serde_json::from_str(&body) {
            Ok(resp) => resp,
            Err(e) => {
                return Err(anyhow!("Failed to parse Diia API response: {}, Body: {}", e, body).into());
            }
        };

        info!("Point 3 - Got deeplink");
        Ok(Json(Response {
            deeplink: api_response.deeplink,
        }))
    } else {
        let error_body = response.text().await?;
        Err(anyhow!(
            "Diia host returned error status {}: {}",
            response.status(),
            error_body
        ).into())
    }
}