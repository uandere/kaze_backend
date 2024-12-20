use axum::extract::{State, Json};
use serde::{Deserialize, Serialize};
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
) -> Json<DiiaResponse> {
    Json(DiiaResponse { success: true })
}
