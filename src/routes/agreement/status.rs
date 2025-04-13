use axum::{extract::{Query, State}, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    commands::server::ServerState,
    utils::{
        cache::AgreementProposalKey,
        db::{get_agreement, get_agreements_for_tenant_and_landlord},
        server_error::ServerError,
    },
};

/// Represents the status of the agreement.
#[derive(Serialize)]
pub enum AgreementStatus {
    NotInitiated,
    Initiated { confirmed_by: String },
    Generated,
    HalfSigned { signed_by: String },
    Signed,
}

#[derive(Deserialize)]
pub struct Payload {
    tenant_id: String,
    landlord_id: String,
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
        date,
    }): Query<Payload>,
) -> Result<Json<Response>, ServerError> {
    let mut status = AgreementStatus::NotInitiated;

    // if the agreement is in the cache - upgrading status to Initiated / Generated / HalfSigned
    if let Some(val) = state
        .cache
        .get(&AgreementProposalKey {
            tenant_id: tenant_id.clone(),
            landlord_id: landlord_id.clone(),
        })
        .await
    {
        if val.landlord_confirmed || val.tenant_confirmed {
            if val.landlord_confirmed && val.tenant_confirmed {
                status = AgreementStatus::Generated;
            } else if val.landlord_confirmed {
                status = AgreementStatus::Initiated {
                    confirmed_by: landlord_id.clone(),
                }
            } else {
                status = AgreementStatus::Initiated {
                    confirmed_by: tenant_id.clone(),
                }
            }
        }

        if val.landlord_signed || val.tenant_signed {
            if val.landlord_signed {
                status = AgreementStatus::HalfSigned {
                    signed_by: landlord_id.clone(),
                };
            } else {
                status = AgreementStatus::HalfSigned {
                    signed_by: tenant_id.clone(),
                };
            }
        }
    }

    // if the agreement is in the database, changing the status to Signed
    if let Some(date) = date {
        let agreement = get_agreement(&state.db_pool, &tenant_id, &landlord_id, &date).await;
        if let Ok(Some(_)) = agreement {
            status = AgreementStatus::Signed;
        }
    } else {
        // checking the latest one
        let agreements = get_agreements_for_tenant_and_landlord(
            &state.db_pool,
            &tenant_id.clone(),
            &landlord_id.clone(),
        )
        .await?;
        if !agreements.is_empty() {
            status = AgreementStatus::Signed;
        }
    }

    Ok(Json(Response { status }))
}
