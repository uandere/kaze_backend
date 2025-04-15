use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::{Query, State}, response::Response};
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
    pub _uid: Option<String>,
}

/// Retuns the data about the latest rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(payload): Query<Payload>,
) -> Result<Response, ServerError> {
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

    let pdf = s3::get_agreement_pdf(
        &state,
        Arc::new(AgreementProposalKey {
            tenant_id: payload.tenant_id,
            landlord_id: payload.landlord_id,
            housing_id: payload.housing_id,
        }),
    )
    .await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"agreement.pdf\"",
        )
        .body(axum::body::Body::from(pdf))
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(response)
}
