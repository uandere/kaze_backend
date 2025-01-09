use axum::extract::{Json, Multipart, State};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::commands::subcommands::server::ServerState;

#[derive(Deserialize)]
pub struct DiiaPayload {

}

#[derive(Serialize)]
pub struct DiiaResponse {
    success: bool
}


pub async fn diia(
    State(_state): State<ServerState>,
    mut multipart: Multipart,
) -> Json<DiiaResponse> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("<unnamed>").to_string();
        let file_name = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());
        let value = field.bytes().await.unwrap_or_else(|_| vec![].into());

        info!("Field Name: {}", name);
        if let Some(file_name) = file_name {
            info!("File Name: {}", file_name);
        }
        if let Some(content_type) = content_type {
            info!("Content Type: {}", content_type);
        }
        info!("Field Value (bytes): {:?}", &value[..std::cmp::min(value.len(), 50)]);
    }

    Json(DiiaResponse {
        success: true,
    })
}
