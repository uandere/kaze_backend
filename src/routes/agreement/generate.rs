use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use moka::ops::compute::Op;
use serde::{Deserialize, Serialize};
use tracing::info;
use typst_pdf::PdfOptions;

use crate::{
    commands::server::ServerState,
    utils::{
        agreement::{generate, HousingData, OwneshipData, RentData, RequisitesData},
        cache::{AgreementProposalKey, AgreementProposalValue},
        db, s3,
        server_error::ServerError,
        typst::TypstWrapperWorld,
        verify_jwt::verify_jwt,
    },
};

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    /// The housing data, like address or area.
    #[serde(default)]
    pub housing_data: HousingData,
    /// The rental data, like meter readings or monthly price.
    #[serde(default)]
    pub rent_data: RentData,
    /// The requisites data, like phone numbers and emails.
    #[serde(default)]
    pub requisites_data: RequisitesData,
    /// The ownership data, like ownership record number and date.
    #[serde(default)]
    pub ownership_data: OwneshipData,
}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<Payload>,
) -> Result<(), ServerError> {
    // TODO: store request of generation in the database and match them with other party's request.
    let token = bearer.token();
    let uid = verify_jwt(token, &state).await?;
    if uid != payload.landlord_id || uid != payload.tenant_id {
        return Err(anyhow!(
            "you are not authorized to perform this action: you're not a landlord or a tenant"
        )
        .into());
    }

    info!("Cache before /generate: {:?}", state.cache);

    // If we got two confirmations, actually generating a file
    let result = state
        .cache
        .entry(AgreementProposalKey {
            tenant_id: payload.tenant_id.clone(),
            landlord_id: payload.tenant_id.clone(),
        })
        .and_compute_with(|entry| {
            let op = match entry {
                Some(entry) => {
                    if uid == payload.tenant_id {
                        Op::Put(Arc::new(AgreementProposalValue {
                            tenant_confirmed: true,
                            ..*entry.into_value().as_ref()
                        }))
                    } else {
                        Op::Put(Arc::new(AgreementProposalValue {
                            landlord_confirmed: true,
                            ..*entry.into_value().as_ref()
                        }))
                    }
                }
                None => Op::Nop,
            };

            std::future::ready(op)
        })
        .await;

    info!("Cache after /generate: {:?}", state.cache);

    match result {
        // if both parties agreed => generating the agreement.
        moka::ops::compute::CompResult::ReplacedWith(entry) => {
            let val = entry.value();
            if !(val.landlord_confirmed && val.tenant_confirmed) {
                return Ok(());
            }
        }
        _ => {
            return Ok(());
        }
    }

    // Here we know for sure that both parties confirmed

    // Trying to get data from the database
    let tenant_data = db::get_document_unit_from_db(&state.db_pool, &payload.tenant_id)
        .await
        .ok_or(anyhow!("cannot get tenant data from db"))?;

    // Similarly for landlord data
    let landlord_data = db::get_document_unit_from_db(&state.db_pool, &payload.landlord_id)
        .await
        .ok_or(anyhow!("cannot get tenant data from db"))?;

    let typst_code = generate(
        &state,
        tenant_data,
        landlord_data,
        payload.housing_data,
        payload.rent_data,
        payload.requisites_data,
        payload.ownership_data,
    )
    .await?;

    let world = TypstWrapperWorld::new("./".to_owned(), typst_code.to_owned());

    let document = typst::compile(&world)
        .output
        .map_err(|_| anyhow!("cannot compile Typst document"))?;

    let pdf = typst_pdf::pdf(&document, &PdfOptions::default()).expect("Error exporting PDF");

    // writing a file to S3 with a corresponding key
    s3::upload_agreement_pdf(
        &state,
        pdf,
        Arc::new(AgreementProposalKey {
            tenant_id: payload.tenant_id,
            landlord_id: payload.landlord_id,
        }),
    )
    .await?;

    Ok(())
}
