use axum::extract::{State, Json};
use serde::{Deserialize, Serialize};
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct Payload {    
    
}

#[derive(Serialize)]
pub struct Response {
    success: bool
}


pub async fn handler(
    State(_state): State<ServerState>,
    _payload: Json<Payload>) -> Json<Response> {
    Json(Response { success: true })
}
