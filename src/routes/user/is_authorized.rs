use axum::extract::{State, Json};
use serde::{Deserialize, Serialize};
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct GeneratePayload {    
    
}

#[derive(Serialize)]
pub struct GenerateResponse {
    success: bool
}


pub async fn handler(
    State(_state): State<ServerState>,
    _payload: Json<GeneratePayload>) -> Json<GenerateResponse> {
    Json(GenerateResponse { success: true })
}
