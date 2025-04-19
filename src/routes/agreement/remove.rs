use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    commands::server::ServerState,
    utils::{
        cache::AgreementProposalKey,
        db::{delete_latest_agreement, remove_signature_entry},
        s3::{get_key_for_s3, get_signature_key_for_s3},
        server_error::ServerError,
        verify_jwt::verify_jwt,
    },
};

/// The input payload to remove an agreement
#[derive(Deserialize)]
pub struct RemoveAgreementPayload {
    pub tenant_id: String,
    pub landlord_id: String,
    pub housing_id: String,

    /// backdoor for testing
    pub _uid: Option<String>,
}

/// Response for `remove_agreement` endpoint
#[derive(Serialize)]
pub struct RemoveAgreementResponse {
    pub success: bool,
}

/// This route completely removes the agreement from DB + S3.
///
/// If the request is made by either `tenant_id` or `landlord_id`,
/// we proceed with removal. Otherwise, we fail.
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<RemoveAgreementPayload>,
) -> Result<Response, ServerError> {
    // 1) figure out who is calling
    let uid = if let Some(_uid) = payload._uid {
        _uid
    } else {
        verify_jwt(bearer.token(), &state).await?
    };

    // 2) check that the caller is either the tenant or landlord
    if uid != payload.tenant_id && uid != payload.landlord_id {
        return Err(anyhow!(
            "you are not authorized to remove this agreement: you're not a landlord or a tenant"
        ).into());
    }

    // 3) remove from DB
    //   - remove the row in `agreements` table
    //   - remove the row in `signatures` table
    //     (and return the removed signature if it existed)
    let _deleted_agreement = delete_latest_agreement(
        &state.db_pool,
        &payload.tenant_id,
        &payload.landlord_id,
        &payload.housing_id,
    )
    .await?;

    let _removed_signatures = remove_signature_entry(
        &state.db_pool,
        &payload.tenant_id,
        &payload.landlord_id,
        &payload.housing_id,
    )
    .await?;

    // (If you have a `CACHE`, and you want to remove from cache as well):
    state.cache.invalidate(&AgreementProposalKey {
        tenant_id: payload.tenant_id.clone(),
        landlord_id: payload.landlord_id.clone(),
        housing_id: payload.housing_id.clone(),
    }).await;

    // 4) remove from S3
    let pdf_key = get_key_for_s3(Arc::new(AgreementProposalKey {
        tenant_id: payload.tenant_id.clone(),
        landlord_id: payload.landlord_id.clone(),
        housing_id: payload.housing_id.clone(),
    }));

    let sig_key = get_signature_key_for_s3(Arc::new(AgreementProposalKey {
        tenant_id: payload.tenant_id.clone(),
        landlord_id: payload.landlord_id.clone(),
        housing_id: payload.housing_id.clone(),
    }));

    info!("Removing from S3: {pdf_key} and {sig_key}");

    // best-effort remove PDF
    let _ = state
        .aws_s3_client
        .delete_object()
        .bucket(&state.s3_bucket_name)
        .key(pdf_key)
        .send()
        .await;

    // best-effort remove P7S signature
    let _ = state
        .aws_s3_client
        .delete_object()
        .bucket(&state.s3_bucket_name)
        .key(sig_key)
        .send()
        .await;

    // 5) respond with success
    Ok((StatusCode::OK, Json(RemoveAgreementResponse { success: true })).into_response())
}
