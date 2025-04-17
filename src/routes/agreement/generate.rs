use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use moka::ops::compute::Op;
use serde::{Deserialize, Serialize};
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

#[derive(Deserialize)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    pub housing_id: String,
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

    /// This is a backdoor for testing purposes
    pub _uid: Option<String>,
}

#[derive(Serialize)]
pub struct Response {}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<Payload>,
) -> Result<Json<Response>, ServerError> {
    let uid = if let Some(_uid) = payload._uid {
        _uid
    } else {
        let token = bearer.token();
        verify_jwt(token, &state).await?
    };

    if !(uid == payload.landlord_id || uid == payload.tenant_id) {
        return Err(anyhow!(
            "you are not authorized to perform this action: you're not a landlord or a tenant"
        )
        .into());
    }

    let tenant_data = db::get_document_unit_from_db(&state.db_pool, &payload.tenant_id).await?;

    let landlord_data = db::get_document_unit_from_db(&state.db_pool, &payload.landlord_id).await?;

    let result = state
        .cache
        .entry(AgreementProposalKey {
            tenant_id: payload.tenant_id.clone(),
            landlord_id: payload.landlord_id.clone(),
            housing_id: payload.housing_id.clone(),
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
                None => {
                    if uid == payload.tenant_id {
                        Op::Put(Arc::new(AgreementProposalValue {
                            tenant_confirmed: true,
                            ..Default::default()
                        }))
                    } else {
                        Op::Put(Arc::new(AgreementProposalValue {
                            landlord_confirmed: true,
                            ..Default::default()
                        }))
                    }
                }
            };

            std::future::ready(op)
        })
        .await;

    match result {
        moka::ops::compute::CompResult::ReplacedWith(entry) => {
            let val = entry.value();
            if !(val.landlord_confirmed && val.tenant_confirmed) {
                return Ok(Json(Response {}));
            }
        }
        _ => {
            return Ok(Json(Response {}));
        }
    }

    // If we got two confirmations, actually generating a file
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

    let pdf = tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<u8>> {
        let world = TypstWrapperWorld::new("./".to_owned(), typst_code);
        let document = typst::compile(&world)
            .output
            .map_err(|e| anyhow!("cannot compile Typst document {:?}", e))?;
        Ok(typst_pdf::pdf(&document, &PdfOptions::default())
            .expect("Error exporting PDF"))
    })
    .await??;

    // writing a file to S3 with a corresponding key
    s3::upload_agreement_pdf(
        &state,
        pdf,
        Arc::new(AgreementProposalKey {
            tenant_id: payload.tenant_id,
            landlord_id: payload.landlord_id,
            housing_id: payload.housing_id,
        }),
    )
    .await?;

    Ok(Json(Response {}))
}
