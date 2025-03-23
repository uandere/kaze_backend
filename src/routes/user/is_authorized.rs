use crate::commands::server::ServerState;
use axum::extract::{Json, State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GeneratePayload {}

#[derive(Serialize)]
pub struct GenerateResponse {
    success: bool,
}

pub async fn handler(
    State(_state): State<ServerState>,
    _payload: Json<GeneratePayload>,
) -> Json<GenerateResponse> {
    Json(GenerateResponse { success: true })
}
