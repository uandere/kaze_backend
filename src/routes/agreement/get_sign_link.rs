use std::{ptr::null_mut, sync::Arc};

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
use uuid::Uuid;

use crate::{
    commands::server::ServerState,
    utils::{
        cache::AgreementProposalKey,
        eusign::{EU_CTX_HASH_ALGO_SHA256, EU_ERROR_NONE, G_P_IFACE},
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
    pub housing_id: String,
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
    pub housing_id: String,

    /// This is a backdoor for testing purposes
    pub _uid: Option<String>,
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
    let uid = if let Some(_uid) = payload._uid {
        _uid
    } else {
        let token = bearer.token();
        verify_jwt(token, &state).await?
    };

    if !(uid == payload.landlord_id || uid == payload.tenant_id) {
        return Err(anyhow!(
            "you are not authorized to perform this action: you're not a landlord or a tenant"
        )
        .into());
    }

    // checking whether users confirmed the generation
    match state
        .cache
        .get(&AgreementProposalKey {
            tenant_id: payload.tenant_id.clone(),
            landlord_id: payload.landlord_id.clone(),
            housing_id: payload.housing_id.clone(),
        })
        .await
    {
        Some(entry) => {
            if !(entry.landlord_confirmed && entry.tenant_confirmed) {
                return Err(anyhow!(
                    "cannot get a sign link: either tenant or a landlord didn't confirm the generation"
                )
                .into());
            }
        }
        None => {
            return Err(anyhow!(
                "cannot get a sign link: either tenant or a landlord didn't confirm the generation"
            )
            .into());
        }
    }

    // getting the file to generate signed hash
    let mut pdf = get_agreement_pdf(
        &state,
        Arc::new(AgreementProposalKey {
            tenant_id: payload.tenant_id.clone(),
            landlord_id: payload.landlord_id.clone(),
            housing_id: payload.housing_id.clone(),
        }),
    )
    .await?;

    // generating the hash
    let base64_hash = unsafe {
        // 1) allocate pointers
        let mut p_hash = null_mut();
        let mut hash_len = 0;
    
        // 2) call the C hash
        let hash_fn = (*G_P_IFACE).CtxHashData
            .ok_or(anyhow!("couldn't get the CtxHashData function from EUSign"))?;
        let err = hash_fn(
            state.ctx.lib_ctx as *mut _,
            EU_CTX_HASH_ALGO_SHA256.into(),
            null_mut(),
            0,
            pdf.as_mut_ptr(),
            pdf.len().try_into()?,
            &mut p_hash,
            &mut hash_len,
        );
        if err != EU_ERROR_NONE as u64 {
            return Err(EUSignError(err).into());
        }
    
        // 3) copy into a Rust Vec<u8>
        let hash_slice = std::slice::from_raw_parts(p_hash, hash_len as usize);
        let hash_bytes = hash_slice.to_vec();
    
        // 4) free the C buffer with the proper call
        let free_fn = (*G_P_IFACE).CtxFreeMemory
            .ok_or(anyhow!("couldn't get the CtxFreeMemory function from EUSign"))?;
        free_fn(state.ctx.lib_ctx as *mut _, p_hash);
    
        // 5) base64‑encode your Rust buffer
        STANDARD.encode(&hash_bytes)
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
