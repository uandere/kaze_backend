use std::str::from_utf8;

use crate::{
    commands::server::ServerState,
    utils::{db, eusign::*, server_error::ServerError},
};
use anyhow::{anyhow, Context};
use axum::extract::{Json, Multipart, State};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
pub struct Payload {}

#[derive(Serialize)]
pub struct Response {
    success: bool,
}

/// This route handles encrypted packages of data that come from Diia servers.
///
/// For now, the pipeline of handling the data is:
/// 1. Decrypting the data using EUSignCP library.
/// 2. Verifying that the data is signed by Diia public certificate.
/// 3. Storing the data inside the database.
pub async fn handler(
    State(state): State<ServerState>,
    mut multipart: Multipart,
) -> Result<Json<Response>, ServerError> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        // 1) GET THE DATA
        let name = field.name().unwrap_or("<unnamed>").to_string();

        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.txt", name));
        let content_type = field.content_type().map(|s| s.to_string());
        let value = field.bytes().await.unwrap_or_else(|_| vec![].into());

        info!("Field Name: {}", name);
        info!("File Name: {}", file_name);
        if let Some(content_type) = content_type {
            info!("Content Type: {}", content_type);
        }
        info!(
            "Field Value (bytes): {:?}",
            &value[..std::cmp::min(value.len(), 50)]
        );

        if name != "encodeData" {
            continue;
        }

        let customer_data = from_utf8(&value)?;

        // 2) DECRYPT THE DATA
        let result = unsafe { decrypt_customer_data(&state, customer_data)? };

        info!("The result of the decryption: {}", result);

        // 3) PARSE AND STORE THE DATA
        // Deserializing using serde
        let result: DecryptionResult = serde_json::from_str(&result)?;

        // Getting user_id and random seed
        let mut request_iter = result.request_id.split_whitespace();
        let user_id = request_iter.next().context("cannot get user ID")?;
        // Checking if there's seed
        let _seed = request_iter.next().context("cannot get random seed")?;

        // Getting the actual passport data
        let data = result.data;

        let taxpayer_card = data
            .taxpayer_card
            .into_iter()
            .next()
            .ok_or(anyhow!("No taxpayer card found"))?;

        let internal_passport = data
            .internal_passport
            .into_iter()
            .next()
            .ok_or(anyhow!("No internal passport found"))?;

        let unit = DocumentUnit {
            taxpayer_card,
            internal_passport,
        };

        // Store in database
        db::store_document_unit(&state.db_pool, user_id, &unit).await?;

        info!("Added user with id={user_id} to the database!");
    }

    Ok(Json(Response { success: true }))
}
