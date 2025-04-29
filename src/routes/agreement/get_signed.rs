use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    response::Response,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use http::{header, StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    commands::server::ServerState,
    utils::{cache::AgreementProposalKey, s3, server_error::ServerError, verify_jwt::verify_jwt},
};

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    pub housing_id: String,

    #[cfg(feature = "dev")]
    pub _uid: Option<String>,
}

/// Retuns the data about the latest rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(payload): Query<Payload>,
) -> Result<Response, ServerError> {
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

    let key = Arc::new(AgreementProposalKey {
        tenant_id: payload.tenant_id.clone(),
        landlord_id: payload.landlord_id.clone(),
        housing_id: payload.housing_id.clone(),
    });

    let pdf_signed = s3::get_agreement_ps7(&state, key.clone()).await?;

    let filename = s3::get_signature_key_for_s3(key);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pkcs7-signature")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}.p7s\""),
        )
        .body(axum::body::Body::from(pdf_signed))
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(response)
}
