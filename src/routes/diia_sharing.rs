use axum::extract::{Json, Multipart, State};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct DiiaPayload {}

#[derive(Serialize)]
pub struct DiiaResponse {
    success: bool,
}

/// This route handles encrypted packages of data from that come from Diia servers.
/// 
/// For now, the pipeline of handling the data is:
/// 1. Decrypting the data using EUSignCP library.
/// 2. Verifying that the data is signed by Diia public certificate.
/// 3. Storing the data inside the cache.
pub async fn diia_sharing(
    State(_state): State<ServerState>,
    mut multipart: Multipart,
) -> Json<DiiaResponse> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        // 1) GET THE DATA
        let name = field.name().unwrap_or("<unnamed>").to_string();

        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| format!("{}.txt", name));
        let content_type = field.content_type().map(|s| s.to_string());
        let value = field.bytes().await.unwrap_or_else(|_| vec![].into());

        info!("Field Name: {}", name);
        info!("File Name: {}", file_name);
        if let Some(content_type) = content_type {
            info!("Content Type: {}", content_type);
        }
        info!("Field Value (bytes): {:?}", &value[..std::cmp::min(value.len(), 50)]);

        if name != "encodeData" {
            continue;
        }

        // 2) DECRYPT THE DATA
        

        // 3) STORE THE DATA
        
    }

    Json(DiiaResponse {
        success: true,
    })
}
