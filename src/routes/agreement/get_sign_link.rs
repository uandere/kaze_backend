use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    commands::server::ServerState,
    utils::{
        s3::get_agreement_pdf, server_error::ServerError, verify_jwt::verify_jwt
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashedFile {
    pub file_name: String,
    pub file_hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashedFilesSigning {
    pub hashed_files: Vec<HashedFile>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestData {
    pub hashed_files_signing: HashedFilesSigning,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignHashRequest {
    pub offer_id: String,
    pub return_link: String,
    pub request_id: String,
    pub sign_algo: Option<String>,
    pub data: RequestData,
}

#[derive(Serialize, Deserialize)]
pub struct SignHashRequestId {
    pub tenant_id: Uuid,
    pub landlord_id: Uuid,
    pub signed_by: Uuid,
    pub housing_id: Uuid,
    pub seed: Uuid,
}

#[derive(Deserialize)]
struct SignHashResponse {
    deeplink: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: Uuid,
    pub landlord_id: Uuid,
    pub housing_id: Uuid,

    /// This is a backdoor for testing purposes
    #[cfg(feature = "dev")]
    pub _uid: Option<Uuid>,
}

#[derive(Serialize)]
pub struct Response {
    pub deeplink: String,
}

/// Generates a Diia Signature deeplink for a user.
/// The deeplink activation through Diia app will trigger the signing of the agreement.
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(payload): Query<Payload>,
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

    if !(uid == payload.landlord_id || uid == payload.tenant_id) {
        return Err(anyhow!(
            "you are not authorized to perform this action: you're not a landlord or a tenant"
        )
        .into());
    }

    // checking whether users confirmed the generation in DB
    // TODO

    // getting the file to generate signed hash
    let pdf = get_agreement_pdf(
        &state,
        payload.tenant_id.clone(),
        payload.landlord_id.clone(),
        payload.housing_id.clone(),
    )
    .await?;

    // generating the hash
    let base64_hash = {
        let mut hasher = Sha256::new();
        hasher.update(pdf);
        let rust_bytes = hasher.finalize().to_vec();
        STANDARD.encode(&rust_bytes)
    };

    let request_id = SignHashRequestId {
        tenant_id: payload.tenant_id,
        landlord_id: payload.landlord_id,
        signed_by: uid,
        housing_id: payload.housing_id,
        seed: Uuid::new_v4(),
    };

    // setting up the request
    let request = SignHashRequest {
        offer_id: state.config.diia.offer_signing_id.clone(),
        return_link: "https://mykaze.org".into(),
        request_id: serde_json::to_string(&request_id)?,
        sign_algo: Some("ECDSA".into()),
        data: RequestData {
            hashed_files_signing: HashedFilesSigning {
                hashed_files: vec![HashedFile {
                    file_name: "agreement.pdf".into(),
                    file_hash: base64_hash,
                }],
            },
        },
    };

    let token = state.diia_session_token.lock().await.clone();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );

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
        .headers(headers)
        .json(&request)
        .send()
        .await?;

    // getting a deeplink and returning it
    if response.status().is_success() {
        let api_response: SignHashResponse = response.json().await?;

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
