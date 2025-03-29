use std::sync::Arc;

use crate::{
    commands::server::ServerState,
    utils::{
        cache::{AgreementProposalKey, AgreementProposalValue},
        server_error::ServerError,
    },
};
use axum::extract::{Json, Multipart, State};
use moka::ops::compute::Op;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RequestId {
    pub tenant_id: String,
    pub landlord_id: String,
    pub signed_by: String,
    pub seed: String,
}

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
    mut _multipart: Multipart,
) -> Result<Json<Response>, ServerError> {
    // TODO
    // 1. Decrypting the hash using EUSignCP library.

    // TODO
    // 2. Getting corresponding agreement PDF from AWS S3.

    // TODO
    // 3. Adding signature to the file.

    // TODO
    // 4. Updating S3 entry.

    // TODO
    // 5. Updating the cache (changing tenant_signed or landlord_singed)

    let request_id = RequestId {
        tenant_id: "1".into(),
        landlord_id: "2".into(),
        signed_by: "1".into(),
        seed: "123".into(),
    };
    let request_id = serde_json::to_string(&request_id)?;
    let RequestId {
        tenant_id,
        landlord_id: _,
        signed_by,
        ..
    } = serde_json::from_str(&request_id)?;

    state
        .cache
        .entry(AgreementProposalKey {
            tenant_id: tenant_id.clone(),
            landlord_id: tenant_id.clone(),
        })
        .and_compute_with(|entry| {
            let op = match entry {
                Some(entry) => {
                    if signed_by == tenant_id {
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
                None => Op::Nop,
            };

            std::future::ready(op)
        }).await;

    Ok(Json(Response { success: true }))
}
