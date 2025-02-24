use anyhow::{anyhow, Context};
use axum::{extract::State, response::Response, Json};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use http::{header, StatusCode};
use serde::{Deserialize, Serialize};
use typst_pdf::PdfOptions;

use crate::{
    commands::server::ServerState,
    utils::{agreement::{generate, HousingData, RentData}, server_error::ServerError, typst::TypstWrapperWorld, verify_jwt::verify_jwt},
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
}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    Json(payload): Json<Payload>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Response, ServerError> {
    // checking authentication
    let token = bearer.token();
    let uid = verify_jwt(token, &state).await?;
    if uid != payload.landlord_id {
        return Err(anyhow!("you are not authorized to perform this action: you're not landlord").into());
    }

    // getting tenant data from the cache
    let tenant_data = state
        .cache
        .get(&payload.tenant_id)
        .await
        .context("no data found for tenant with specified ID")?;

    // getting landlord data from the cache
    let landlord_data = state
        .cache
        .get(&payload.landlord_id)
        .await
        .context("no data found for landlord with specified ID")?;

    let typst_code = generate(&state, tenant_data, landlord_data, payload.housing_data, payload.rent_data).await?;

    let world = TypstWrapperWorld::new("./".to_owned(), typst_code.to_owned());

    let document = typst::compile(&world)
    .output.map_err(|_| anyhow!("cannot compile Typst document"))?;

    let pdf = typst_pdf::pdf(&document, &PdfOptions::default()).expect("Error exporting PDF");

    
    let response = Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "application/pdf")
    .header(header::CONTENT_DISPOSITION, "attachment; filename=\"agreement.pdf\"")
    .body(axum::body::Body::from(pdf))
    .map_err(|e| anyhow!(e.to_string()))?;

    Ok(response)
}
