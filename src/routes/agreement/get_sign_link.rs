use std::{ptr::null_mut, sync::Arc};

use anyhow::anyhow;
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};

use http::{header::{ACCEPT, AUTHORIZATION}, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    commands::server::ServerState,
    utils::{
        cache::AgreementProposalKey,
        eusign::{EU_CTX_HASH_ALGO_GOST34311, EU_ERROR_NONE, G_P_IFACE},
        s3::get_agreement_pdf,
        server_error::{EUSignError, ServerError},
        verify_jwt::verify_jwt,
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
    pub tenant_id: String,
    pub landlord_id: String,
    pub signed_by: String,
    pub seed: Uuid,
}

#[derive(Deserialize)]
struct SignHashResponse {
    deeplink: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,

    /// This is a backdoor for testing purposes
    pub _uid: String,
}

#[derive(Serialize)]
pub struct Response {
    pub deeplink: String,
}

/// Generates a Diia Signature deeplink for a user.
/// The deeplink activation through Diia app will trigger the signing of the agreement.
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<Payload>,
) -> Result<Json<Response>, ServerError> {
    // checking authentication
    // let token = bearer.token();
    // let uid = verify_jwt(token, &state).await?;
    let uid = payload._uid;
    if !(uid == payload.landlord_id || uid == payload.tenant_id) {
        return Err(anyhow!(
            "you are not authorized to perform this action: you're not a landlord or a tenant"
        )
        .into());
    }

    // getting the file to generate signed hash
    let mut pdf = get_agreement_pdf(
        &state,
        Arc::new(AgreementProposalKey {
            tenant_id: payload.tenant_id.clone(),
            landlord_id: payload.landlord_id.clone(),
        }),
    )
    .await?;

    // generating the hash
    let hash_string;

    unsafe {
        let mut hash = std::ptr::null_mut();
        let mut hash_len = 0;

        let ctx_hash_data_func = (*G_P_IFACE)
            .CtxHashData
            .ok_or(anyhow!("couldn't get the hashing function from EUSign"))?;

        let error_code = ctx_hash_data_func(
            state.ctx.lib_ctx as *mut std::ffi::c_void,
            EU_CTX_HASH_ALGO_GOST34311.into(),
            null_mut(),
            0,
            pdf.as_mut_ptr(),
            pdf.len().try_into()?,
            &mut hash,
            &mut hash_len,
        );

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        hash_string = String::from_raw_parts(hash, hash_len.try_into()?, hash_len.try_into()?);
    }

    // encoding hash to base64
    let base64_hash = STANDARD.encode(hash_string);

    let request_id = SignHashRequestId {
        tenant_id: payload.tenant_id,
        landlord_id: payload.landlord_id,
        signed_by: uid,
        seed: Uuid::new_v4(),
    };

    // setting up the request
    let request = SignHashRequest {
        offer_id: state.config.diia.offer_signing_id.clone(),
        return_link: "https://mykaze.org".into(), // TODO: change this to the chat url
        request_id: serde_json::to_string(&request_id)?,
        sign_algo: Some("DSTU".into()),
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
        HeaderValue::from_str(&format!("Bearer {}", token))?,
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
