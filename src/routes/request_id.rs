#![allow(dead_code)]
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetUserInfoPayload {
    user_id: String,
}


#[derive(Serialize)]
pub struct RequestIdResponse {
    name: String,
}

pub async fn diia_user_info(_payload: Query<GetUserInfoPayload>) -> Json<RequestIdResponse> {
    todo!()
}
