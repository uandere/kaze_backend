use anyhow::anyhow;
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::{Deserialize, Serialize};

use crate::{
    commands::server::ServerState,
    utils::{server_error::ServerError, verify_jwt::verify_jwt},
};

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
}

pub struct Response {
    deeplink: String,
}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    // TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<Payload>,
) -> Result<Response, ServerError> {
    // checking authentication
    // let token = bearer.token();
    // let uid = verify_jwt(token, &state).await?;
    // if uid != payload.landlord_id {
    //     return Err(anyhow!("you are not authorized to perform this action: you're not landlord").into());
    // }

    // First try to get data from the database

    Ok(Response {
        deeplink: "".into(),
    })
}
