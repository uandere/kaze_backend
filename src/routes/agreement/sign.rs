use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    commands::server::ServerState,
    utils::server_error::ServerError
};

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    pub _is_landlord: bool,
}

pub struct Response {
    pub deeplink: String,
}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(_state): State<ServerState>,
    // TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(_payload): Json<Payload>,
) -> Result<Response, ServerError> {
    // checking authentication
    // let token = bearer.token();
    // let uid = verify_jwt(token, &state).await?;
    // if uid != payload.landlord_id {
    //     return Err(anyhow!("you are not authorized to perform this action: you're not landlord").into());
    // }



    Ok(Response {
        deeplink: "".into(),
    })
}
