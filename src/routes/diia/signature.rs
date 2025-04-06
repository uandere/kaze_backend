use std::{str::from_utf8, sync::Arc};

use crate::{
    commands::server::ServerState,
    routes::agreement::get_sign_link::SignHashRequestId,
    utils::{cache::AgreementProposalKey, s3::get_agreement_pdf, server_error::ServerError},
};
use anyhow::anyhow;
use axum::extract::{Json, Multipart, State};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use http::HeaderMap;
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
pub struct Response {
    success: bool,
}

/// This route handles signed hashes of the agreement that come from Diia Signature.
///
/// For now, the pipeline of handling the data is:
/// 1. Decrypting the hash using EUSignCP library.
/// 2. Getting corresponding agreement PDF from AWS S3.
/// 3. Adding signature to the file.
/// 4. Updating S3 entry.
/// 5. Updating the cache (changing tenant_signed or landlord_singed)
pub async fn handler(
    State(state): State<ServerState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Response>, ServerError> {
    // TODO
    // 1. Decrypting the hash using EUSignCP library.

    info!("Here 1!");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        info!("Here 2!");
        let name = field.name().unwrap_or("<unnamed>").to_string();

        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.txt", name));
        let content_type = field.content_type().map(|s| s.to_string());
        let value = field.bytes().await.unwrap_or_else(|_| vec![].into());

        info!("Here 3!");

        info!("Field Name: {}", name);
        info!("File Name: {}", file_name);
        if let Some(content_type) = content_type {
            info!("Content Type: {}", content_type);
        }
        info!(
            "Field Value (bytes): {:?}",
            &value[..std::cmp::min(value.len(), 50)]
        );

        info!("Here 4!");

        if name != "encodeData" {
            continue;
        }

        // 2) DECODE THE DATA FROM BASE64
        let result = BASE64_STANDARD.decode(value)?;
        let result = from_utf8(&result)?;

        info!("Here 5!");

        info!("The result of the decoding: {}", result);

        // 2. Getting corresponding agreement PDF from AWS S3.
        let _sign_request_id: SignHashRequestId = serde_json::from_str(
            headers
                .get("X-Document-Request-Trace-Id")
                .ok_or(anyhow!("wasn't able to get sign hash request id header"))?
                .to_str()?,
        )?;

        // let mut pdf = get_agreement_pdf(
        //     &state,
        //     Arc::new(AgreementProposalKey {
        //         tenant_id: payload.tenant_id.clone(),
        //         landlord_id: payload.landlord_id.clone(),
        //     }),
        // )
        // .await?;

        // TODO
        // 3. Adding signature to the file.

        // TODO
        // 4. Updating S3 entry.

        // TODO
        // 5. Updating the cache (changing tenant_signed or landlord_singed)

        // let request_id = SignHashRequestId {
        //     tenant_id: "1".into(),
        //     landlord_id: "2".into(),
        //     signed_by: "1".into(),
        //     seed: uuid::uuid!("12345678-1234-5678-1234-567812345678"),
        // };
        // let request_id = serde_json::to_string(&request_id)?;
        // let SignHashRequestId {
        //     tenant_id,
        //     landlord_id: _,
        //     signed_by,
        //     ..
        // } = serde_json::from_str(&request_id)?;

        // state
        //     .cache
        //     .entry(AgreementProposalKey {
        //         tenant_id: tenant_id.clone(),
        //         landlord_id: tenant_id.clone(),
        //     })
        //     .and_compute_with(|entry| {
        //         let op = match entry {
        //             Some(entry) => {
        //                 if signed_by == tenant_id {
        //                     Op::Put(Arc::new(AgreementProposalValue {
        //                         tenant_signed: true,
        //                         ..*entry.into_value().as_ref()
        //                     }))
        //                 } else {
        //                     Op::Put(Arc::new(AgreementProposalValue {
        //                         landlord_signed: true,
        //                         ..*entry.into_value().as_ref()
        //                     }))
        //                 }
        //             }
        //             None => {
        //                 if signed_by == tenant_id {
        //                     Op::Put(Arc::new(AgreementProposalValue {
        //                         tenant_signed: true,
        //                         ..Default::default()
        //                     }))
        //                 } else {
        //                     Op::Put(Arc::new(AgreementProposalValue {
        //                         landlord_signed: true,
        //                         ..Default::default()
        //                     }))
        //                 }
        //             },
        //         };

        //         std::future::ready(op)
        //     }).await;
    }

    Ok(Json(Response { success: true }))
}
