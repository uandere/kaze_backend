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

/// A handler that returns a user's name if it's authorized using Diia.
/// Returns error otherwise.
pub async fn handler(
    State(state): State<ServerState>,
    payload: Query<Payload>,
) -> Result<Json<Response>, ServerError> {
    let document = db::get_document_unit_from_db(&state.db_pool, &payload.id)
        .await
        .ok_or(anyhow!("cannot get tenant data from db"))?;

    Ok(Json(Response {
        name: document.internal_passport.first_name_ua.clone(),
    }))
}
