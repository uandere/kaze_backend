use crate::{
    commands::server::ServerState,
    utils::{db, server_error::ServerError},
};

use anyhow::anyhow;
use axum::extract::{Json, Query, State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Payload {
    id: String,
}

#[derive(Serialize)]
pub struct Response {
    name: String,
}

/// A handler that returns user's name if user was authorized using Diia Sharing
/// and error otherwise.
pub async fn handler(
    State(state): State<ServerState>,
    payload: Query<Payload>,
) -> Result<Json<Response>, ServerError> {
    match db::get_document_unit_from_db(&state.db_pool, &payload.id).await {
        Ok(doc) => Ok(Json(Response { name: doc.internal_passport.first_name_ua.clone() })),
        Err(_) => {
            Err(anyhow!("user is not authorized with Diia").into())
        },
    }
}
