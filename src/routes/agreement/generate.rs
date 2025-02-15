use anyhow::Context;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    commands::server::ServerState,
    utils::{agreement::{generate, HousingData, RentData}, server_error::ServerError},
};


#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    pub housing_data: HousingData,
    pub rent_data: RentData
}

#[derive(Serialize)]
pub struct Response {
    pub pdf: String,
}

/// Generates rental ageement between tenant and landlord.
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ServerState>,
    Json(payload): Json<Payload>,
) -> Result<Json<Response>, ServerError> {


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

    // generating agreement with all the data
    // let pdf = generate(tenant_data, landlord_data, housing_data).await;

    // TODO: not pass default
    let pdf = generate(&state, tenant_data, landlord_data, Default::default(), Default::default()).await?;

    tokio::fs::write("output.typ", pdf.clone()).await?;

    Ok(Json(Response { pdf }))
}
