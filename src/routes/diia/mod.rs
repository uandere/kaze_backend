use axum::extract::{State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::commands::subcommands::server::ServerState;

#[derive(Deserialize)]
pub struct DiiaPayload {

}

#[derive(Serialize)]
pub struct DiiaResponse {
    success: bool
}


pub async fn diia(
    State(_state): State<ServerState>,
    Json(payload): Json<Value>,
) -> Json<DiiaResponse> {
    println!("Received payload: {}", payload);
    Json(DiiaResponse { success: true })
}
