use anyhow::{anyhow, Context};
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
        agreement::{generate, HousingData, RentData},
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
    let tenant_data = match db::get_document_unit_from_db(&state.db_pool, &payload.tenant_id).await
    {
        Some(data) => data,
        None => {
            // If not in database, try the cache as fallback during transition period
            state
                .cache
                .get(&payload.tenant_id)
                .await
                .context("no data found for tenant with specified ID")?
        }
    };

    // Similarly for landlord data
    let landlord_data =
        match db::get_document_unit_from_db(&state.db_pool, &payload.landlord_id).await {
            Some(data) => data,
            None => {
                // If not in database, try the cache as fallback
                state
                    .cache
                    .get(&payload.landlord_id)
                    .await
                    .context("no data found for landlord with specified ID")?
            }
        };

    let typst_code = generate(
        &state,
        tenant_data,
        landlord_data,
        payload.housing_data,
        payload.rent_data,
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
