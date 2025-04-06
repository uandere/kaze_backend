use std::{ffi::c_char, ptr::null_mut, str::from_utf8, sync::Arc};

use crate::{
    commands::server::ServerState,
    routes::agreement::get_sign_link::SignHashRequestId,
    utils::{
        cache::AgreementProposalKey, eusign::G_P_IFACE, s3::get_agreement_pdf,
        server_error::ServerError,
    },
};
use anyhow::{anyhow, Context};
use axum::extract::{Json, Multipart, State};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize)]
pub struct Response {
    success: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignedItems {
    pub name: String,
    pub signature: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignedHash {
    pub signed_items: Vec<SignedItems>,
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
        let mut result: SignedHash = serde_json::from_str(result)?;

        info!("Here 5!");

        info!("The result of the decoding: {:?}", result);

        // 2. Getting corresponding agreement PDF from AWS S3.
        let SignHashRequestId {
            tenant_id,
            landlord_id,
            signed_by,
            ..
        } = serde_json::from_str(
            headers
                .get("X-Document-Request-Trace-Id")
                .ok_or(anyhow!("wasn't able to get sign hash request id header"))?
                .to_str()?,
        )?;

        let pdf = get_agreement_pdf(
            &state,
            Arc::new(AgreementProposalKey {
                tenant_id: tenant_id.clone(),
                landlord_id: landlord_id.clone(),
            }),
        )
        .await?;

        let cache_entry = state
            .cache
            .get(&AgreementProposalKey {
                tenant_id: tenant_id.clone(),
                landlord_id: landlord_id.clone(),
            })
            .await
            .ok_or(anyhow!("cannot sign: agreement doesn't exist"))?;

        // if other party already signed, incrementing index
        let signature_idx = if cache_entry.landlord_signed || cache_entry.tenant_signed {
            1_u64
        } else {
            0
        };

        let signature = unsafe {
            result
                .signed_items
                .first_mut()
                .context("signatory didn't sign any file")?
                .signature
                .as_bytes_mut()
        };
        let mut cert_info = null_mut();
        let mut cert = null_mut();
        let cert_size = null_mut();

        unsafe {
            let ctx_get_signer_info = (*G_P_IFACE)
                .CtxGetSignerInfo
                .context("wasn't able to get get_signer_info handler")?;

            ctx_get_signer_info(
                state.ctx.lib_ctx as *mut std::ffi::c_void,
                signature_idx,
                signature.as_mut_ptr(),
                signature.len().try_into()?,
                &mut cert_info,
                &mut cert,
                cert_size,
            );
        }

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
