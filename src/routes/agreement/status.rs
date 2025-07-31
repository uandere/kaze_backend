use axum::{
    extract::{Query, State},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    commands::server::ServerState,
    utils::{
        db::{get_agreement, get_agreements_for_tenant_and_landlord},
        server_error::ServerError,
    },
};

/// Represents the status of the agreement.
/// This structure is intended to be read by the frontend.
#[derive(Serialize)]
pub enum AgreementStatus {
    NotInitiated,
    Initiated { by: Uuid },
    Rejected {by: Uuid},
    Generated,
    HalfSigned { by: Uuid },
    Signed,

}

#[derive(Deserialize)]
pub struct Payload {
    tenant_id: String,
    landlord_id: String,
    housing_id: String,
    date: Option<NaiveDate>,
}

#[derive(Serialize)]
pub struct Response {
    status: AgreementStatus,
}

/// Returns the status of the agreement between tenant and landlord.
/// The date field is optional: if not passed, the latest agreement will be considered.
pub async fn handler(
    State(state): State<ServerState>,
    Query(Payload {
        tenant_id,
        landlord_id,
        housing_id,
        date,
    }): Query<Payload>,
) -> Result<Json<Response>, ServerError> {
    // TODO: getting the status from the DB column
    todo!()
}
