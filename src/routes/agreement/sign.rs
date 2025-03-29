use axum::{extract::State, Json};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use serde::{Deserialize, Serialize};
use anyhow::anyhow;

use crate::{commands::server::ServerState, utils::{server_error::ServerError, verify_jwt::verify_jwt}};

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
}

pub struct Response {
    pub deeplink: String,
}

/// Generates a deeplink for Diia Signature.
/// The deeplink activation through Diia app will trigger the signing process.
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<Payload>,
) -> Result<Response, ServerError> {
    // checking authentication
    let token = bearer.token();
    let uid = verify_jwt(token, &state).await?;
    if uid != payload.landlord_id {
        return Err(anyhow!("you are not authorized to perform this action: you're not a landlord or a tenant").into());
    }

    // getting the file to generate signed hash

    // generating hash

    // getting the deeplink
    Ok(Response {
        deeplink: "".into(),
    })
}
