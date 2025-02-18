use std::sync::Arc;

use anyhow::{anyhow, Context};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use typst_pdf::PdfOptions;

use crate::{
    commands::server::ServerState,
    utils::{agreement::{generate, HousingData, RentData}, eusign::DocumentUnit, server_error::ServerError, typst::TypstWrapperWorld},
};


#[derive(Deserialize, Serialize, Default)]
pub struct Payload {
    pub tenant_id: String,
    pub landlord_id: String,
    #[serde(default)]
    pub housing_data: HousingData,
    #[serde(default)]
    pub rent_data: RentData,
    pub _tenant: Option<DocumentUnit>,
    pub _landlord: Option<DocumentUnit>,
}

#[derive(Serialize)]
pub struct Response {
    #[serde(with="base64")]
    pub pdf: Vec<u8>,
}

/// Generates rental ageement between tenant and landlord.
// #[axum::debug_handler]
pub async fn handler(
    State(state): State<ServerState>,
    Json(payload): Json<Payload>,
) -> Result<Json<Response>, ServerError> {
    // getting tenant data from the cache
    let mut tenant_data = state
        .cache
        .get(&payload.tenant_id)
        .await
        .context("no data found for tenant with specified ID")?;

    // getting landlord data from the cache
    let mut landlord_data = state
        .cache
        .get(&payload.landlord_id)
        .await
        .context("no data found for landlord with specified ID")?;

    // generating agreement with all the data
    // let pdf = generate(tenant_data, landlord_data, housing_data).await;

    // === This is demo-only code === //
    if let Some (_tenant) = payload._tenant {
        tenant_data = Arc::new(_tenant);
    }

    if let Some (_landlord) = payload._landlord {
        landlord_data = Arc::new(_landlord);
    }
    // ============================== //

    let typst_code = generate(&state, tenant_data, landlord_data, payload.housing_data, payload.rent_data).await?;

    let world = TypstWrapperWorld::new("./".to_owned(), typst_code.to_owned());

    let document = typst::compile(&world)
    .output.map_err(|_| anyhow!("cannot compile Typst document"))?;

    let pdf = typst_pdf::pdf(&document, &PdfOptions::default()).expect("Error exporting PDF");

    Ok(Json(Response { pdf }))
}

mod base64 {
    use serde::Serialize;
    use serde::Serializer;
    use base64::{engine::general_purpose::STANDARD, Engine as _};


    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = STANDARD.encode(v);
        String::serialize(&base64, s)
    }
    
    // pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
    //     let base64 = String::deserialize(d)?;
    //     base64::decode(base64.as_bytes())
    //         .map_err(|e| serde::de::Error::custom(e))
    // }
}
