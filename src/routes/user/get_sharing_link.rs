use crate::{commands::server::ServerState, utils::server_error::ServerError};
use anyhow::anyhow;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
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
    // let token = bearer.token();
    // let uid = verify_jwt(token, &state).await?;
    let uid = payload._uid;


    let request_id = DiiaSharingRequestId {
        uid,
        seed: Uuid::new_v4(),
    };

    // setting up the request
    let request = DiiaSharingRequest {
        offer_id: state.config.diia.offer_id.clone(),
        return_link: "https://mykaze.org".into(), // TODO: change this to the corresponding url
        request_id: serde_json::to_string(&request_id)?,
    };

    // setting up endpoint
    let base_url = format!("{}/api/v2/acquirers/branch", state.config.diia.host);
    let endpoint = format!(
        "{}/{}/offer-request/dynamic",
        base_url, state.config.diia.branch_id
    );

    // sending the request
    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .header("accept", "application/json")
        .header("Authorization", format!("Bearer {}", state.diia_session_token.lock().await.clone()))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    // getting a deeplink and returning it
    if response.status().is_success() {
        let api_response: DiiaSharingResponse = response.json().await?;

        Ok(Json(Response {
            deeplink: api_response.deeplink,
        }))
    } else {
        Err(anyhow!(
            "Diia host returned with status {}: {}",
            response.status(),
            response.text().await?
        )
        .into())
    }
}
