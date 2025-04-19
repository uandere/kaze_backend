use std::{str::from_utf8, sync::Arc};

use crate::{
    commands::server::ServerState,
    routes::agreement::get_sign_link::SignHashRequestId,
    utils::{
        cache::{AgreementProposalKey, AgreementProposalValue},
        db,
        server_error::ServerError,
    },
};
use anyhow::{anyhow, Context};
use axum::extract::{Json, Multipart, State};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use http::HeaderMap;
use moka::ops::compute::Op;
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
/// 1. Getting the signature from the request
/// 2. Adding signature to signatures DB
/// 3. Updating the cache.
pub async fn handler(
    State(state): State<ServerState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<Response>, ServerError> {
    // 1. Decoding the message
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("<unnamed>").to_string();

        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.to_string());
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

        let result = BASE64_STANDARD.decode(value)?;
        let result = from_utf8(&result)?;
        let mut result: SignedHash = serde_json::from_str(result)?;

        // info!("Result: {:?}", result);

        let SignHashRequestId {
            tenant_id,
            landlord_id,
            signed_by,
            housing_id,
            ..
        } = serde_json::from_str(
            headers
                .get("X-Document-Request-Trace-Id")
                .ok_or(anyhow!("wasn't able to get sign hash request id header"))?
                .to_str()?,
        )?;

        let signature = result
            .signed_items
            .pop()
            .context("cannot extract signature")?
            .signature;

        // 2. Updating signatures DB
        db::add_signature(
            &state.db_pool,
            &tenant_id,
            &landlord_id,
            &housing_id,
            &signed_by,
            signature,
        )
        .await?;

    

        // 3. Updating the cache (changing tenant_signed or landlord_singed)
        state
            .cache
            .entry(AgreementProposalKey {
                tenant_id: tenant_id.clone(),
                landlord_id: landlord_id.clone(),
                housing_id: housing_id.clone(),
            })
            .and_compute_with(|entry| {
                let op = match entry {
                    Some(entry) => {
                        // TODO: remove this ===================================
                        if signed_by == tenant_id && signed_by == landlord_id {
                            Op::Put(Arc::new(AgreementProposalValue {
                                tenant_signed: true,
                                landlord_signed: true,
                                ..*entry.into_value().as_ref()
                            }))
                        }
                        // TODO: remove this ===================================
                        else if signed_by == tenant_id {
                            Op::Put(Arc::new(AgreementProposalValue {
                                tenant_signed: true,
                                ..*entry.into_value().as_ref()
                            }))
                        } else {
                            Op::Put(Arc::new(AgreementProposalValue {
                                landlord_signed: true,
                                ..*entry.into_value().as_ref()
                            }))
                        }
                    }
                    None => {
                        if signed_by == tenant_id {
                            Op::Put(Arc::new(AgreementProposalValue {
                                tenant_signed: true,
                                ..Default::default()
                            }))
                        } else {
                            Op::Put(Arc::new(AgreementProposalValue {
                                landlord_signed: true,
                                ..Default::default()
                            }))
                        }
                    }
                };

                std::future::ready(op)
            })
            .await;
    }

    Ok(Json(Response { success: true }))
}
