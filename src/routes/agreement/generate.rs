use anyhow::Context;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    commands::server::ServerState,
    utils::{agreement::generate, server_error::ServerError},
};

#[derive(Deserialize)]
pub struct HousingData {}

#[derive(Deserialize)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    pub housing_data: HousingData,
}

#[derive(Serialize)]
pub struct Response {
    pub pdf: String,
}

/// Generates rental ageement between tenant and landlord.
pub async fn handler(
    State(state): State<ServerState>,
    Json(Payload {
        tenant_id,
        landlord_id,
        housing_data,
    }): Json<Payload>,
) -> Result<Response, ServerError> {
    // getting tenant data from the cache
    let tenant_data = state
        .cache
        .get(&tenant_id)
        .await
        .context("no data found for tenant with specified ID")?;

    // getting landlord data from the cache
    let landlord_data = state
        .cache
        .get(&landlord_id)
        .await
        .context("no data found for landlord with specified ID")?;

    // generating agreement with all the data
    // let pdf = generate(tenant_data, landlord_data, housing_data).await;

    let pdf = generate(&state, tenant_data, landlord_data, housing_data).await?;

    Ok(Response { pdf })
}
