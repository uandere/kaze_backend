use std::sync::Arc;

use anyhow::anyhow;
use axum::{extract::State, response::Response, Json};
use http::{header, StatusCode};
use serde::{Deserialize, Serialize};
use typst_pdf::PdfOptions;

use crate::{
    commands::server::ServerState,
    utils::{
        agreement::{generate, HousingData, OwneshipData, RentData, RequisitesData},
        eusign::DocumentUnit,
        server_error::ServerError,
        typst::TypstWrapperWorld,
    },
};

#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    /// The housing data, like address or area.
    #[serde(default)]
    pub housing_data: HousingData,
    /// The rental data, like meter readings or monthly price.
    #[serde(default)]
    pub rent_data: RentData,
    #[serde(default)]
    pub tenant: DocumentUnit,
    #[serde(default)]
    pub landlord: DocumentUnit,
    #[serde(default)]
    pub requisites_data: RequisitesData,
    #[serde(default)]
    pub ownership_data: OwneshipData,
}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    Json(payload): Json<Payload>,
) -> Result<Response, ServerError> {
    let typst_code = generate(
        &state,
        Arc::new(payload.tenant),
        Arc::new(payload.landlord),
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

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"agreement.pdf\"",
        )
        .body(axum::body::Body::from(pdf))
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(response)
}
