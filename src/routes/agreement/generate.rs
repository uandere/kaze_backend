use anyhow::anyhow;
use axum::{extract::State, response::Response, Json};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use http::{header, StatusCode};
use serde::{Deserialize, Serialize};
use typst_pdf::PdfOptions;

use crate::{
    commands::server::ServerState,
    utils::{
        agreement::{generate, HousingData, OwneshipData, RentData, RequisitesData},
        db,
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
) -> Result<Response, ServerError> {
    // TODO: store request of generation in the database and match them with other party's request.
    let token = bearer.token();
    let uid = verify_jwt(token, &state).await?;
    if uid != payload.landlord_id {
        return Err(
            anyhow!("you are not authorized to perform this action: you're not landlord").into(),
        );
    }

    // First try to get data from the database
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
