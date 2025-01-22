use axum::extract::{Json, Multipart, State};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct DiiaPayload {}

#[derive(Serialize)]
pub struct DiiaResponse {
    success: bool,
}

pub async fn diia_sharing(
    State(_state): State<ServerState>,
    mut multipart: Multipart,
) -> Json<DiiaResponse> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
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

        // Save the field's value to a file
        if let Err(e) = save_to_file(&file_name, &value).await {
            info!("Failed to save file {}: {}", file_name, e);
        }
    }

    Json(DiiaResponse {
        success: true,
    })
}

async fn save_to_file(file_name: &str, data: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(file_name).await?;
    file.write_all(data).await?;
    Ok(())
}
