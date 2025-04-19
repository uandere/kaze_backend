use crate::{
    commands::server::ServerState,
    utils::{db, server_error::ServerError, verify_jwt::verify_jwt},
};
use axum::{
    extract::{Query, State},
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Payload {
    /// This is a backdoor for testing purposes
    pub _uid: Option<String>,
}

#[derive(Serialize)]
pub struct Response {
    success: bool,
}

/// Generates an authorization link for Diia sharing.
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

    if (db::delete_document_unit(&state.db_pool, &uid).await).is_ok() {
        Ok(Json(Response { success: true }))
    } else {
        Ok(Json(Response { success: false }))
    }
}
