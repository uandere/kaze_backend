use crate::{
    commands::server::ServerState,
    utils::{db, server_error::ServerError},
};

use axum::extract::{Json, Query, State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Payload {
    id: String,
}

#[derive(Serialize)]
pub struct Response {
    result: bool,
}

/// A handler that returns true if user was authorized using Diia Sharing
/// and false otherwise.
pub async fn handler(
    State(state): State<ServerState>,
    payload: Query<Payload>,
) -> Result<Json<Response>, ServerError> {
    match db::get_document_unit_from_db(&state.db_pool, &payload.id).await {
        Ok(_) => Ok(Json(Response { result: true })),
        Err(_) => Ok(Json(Response { result: false })),
    }
}
