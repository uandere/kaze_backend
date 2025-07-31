use crate::{
    commands::server::ServerState,
    utils::{db, server_error::ServerError, verify_jwt::verify_jwt},
};
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
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Payload {
    /// This is a backdoor for testing purposes
    #[cfg(feature = "dev")]
    pub _uid: Option<Uuid>,
}

#[derive(Serialize)]
pub struct Response {
    success: bool,
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

    if (db::delete_document_unit(&state.db_pool, uid).await).is_ok() {
        Ok(Json(Response { success: true }))
    } else {
        Ok(Json(Response { success: false }))
    }
}
