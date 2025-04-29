use crate::{
    commands::server::ServerState,
    utils::{server_error::ServerError, verify_jwt::verify_jwt},
};
use anyhow::anyhow;
use axum::{
    extract::State,
    Json,
};

#[cfg(feature = "dev")]
use axum::extract::Query;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Payload {
    /// This is a backdoor for testing purposes
    #[cfg(feature = "dev")]
    pub _uid: Option<String>,
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
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    #[cfg(feature = "dev")] Query(payload): Query<Payload>,
) -> Result<Json<Response>, ServerError> {
    #[cfg(feature = "dev")]
    let uid = if let Some(_uid) = payload._uid {
        _uid
    } else {
        let token = bearer.token();
        verify_jwt(token, &state).await?
    };

    #[cfg(feature = "default")]
    let uid = {
        let token = bearer.token();
        verify_jwt(token, &state).await?
    };

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
        offer_id: state.config.diia.offer_sharing_id.clone(),
        return_link: "https://mykaze.org".into(),
        request_id: request_id_str,
    };

    // setting up endpoint
    let base_url = format!("{}/api/v2/acquirers/branch", state.config.diia.host);
    let endpoint = format!(
        "{}/{}/offer-request/dynamic",
        base_url, state.config.diia.branch_id
    );

    let token = state.diia_session_token.lock().await.clone();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    // sending the request with better error handling
    let client = reqwest::Client::new();
    let response = match client
        .post(&endpoint)
        .headers(headers)
        .json(&request)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            error!("Diia Sharing request failed: {:?}", e);
            return Err(anyhow!("Failed to send request to Diia API: {}", e).into());
        }
    };

    // getting a deeplink and returning it
    if response.status().is_success() {
        let body = response.text().await?;

        let api_response: DiiaSharingResponse = match serde_json::from_str(&body) {
            Ok(resp) => resp,
            Err(e) => {
                return Err(
                    anyhow!("Failed to parse Diia API response: {}, Body: {}", e, body).into(),
                );
            }
        };

        Ok(Json(Response {
            deeplink: api_response.deeplink,
        }))
    } else {
        let error_body = response.text().await?;
        Err(anyhow!("Diia host returned error status: {}", error_body).into())
    }
}
