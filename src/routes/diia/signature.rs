use crate::commands::server::ServerState;
use axum::extract::{Json, State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Payload {}

#[derive(Serialize)]
pub struct Response {
    success: bool,
}

/// This route handles signed (by tenant or landlord) hashes of the agreement.
///
/// For now, the pipeline of handling the data is:
/// 1. Decrypting the data using EUSignCP library.
/// 2. Verifying that the data is signed by Diia public certificate.
/// 3. Storing the data inside the database.
pub async fn handler(State(_state): State<ServerState>, _payload: Json<Payload>) -> Json<Response> {
    Json(Response { success: true })
}
