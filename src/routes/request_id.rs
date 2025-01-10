#![allow(dead_code)]
use axum::{debug_handler, extract::Query, Json};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RequestIdPayload {
    user_id: String,
}


#[derive(Serialize)]
pub struct RequestIdResponse {
    request_id: String,
}

#[debug_handler]
pub async fn request_id(payload: Query<RequestIdPayload>) -> Json<RequestIdResponse> {
    let request_id = Uuid::now_v7().to_string();

    info!("User {} made request {}", payload.user_id, request_id );

    Json(RequestIdResponse { request_id })
}
