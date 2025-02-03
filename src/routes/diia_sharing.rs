use std::{ffi::c_ulong, str::from_utf8};

use crate::{
    commands::server::ServerState,
    utils::{eusign::*, server_error::ServerError},
};
use axum::extract::{Json, Multipart, State};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

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
    State(state): State<ServerState>,
    mut multipart: Multipart,
) -> Result<Json<DiiaResponse>, ServerError> {
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
        // Load the EUSignCP library
        // Load all the necessary things
        unsafe {
            let loaded = EULoad();
            if loaded == 0 {
                // Means it failed
                error!("{}", get_error_message(EU_ERROR_LIBRARY_LOAD.into()));
                std::process::exit(1);
            }

            // 2) Get the interface pointer
            let p_iface = EUGetInterface();
            if p_iface.is_null() {
                error!("{}", get_error_message(EU_ERROR_LIBRARY_LOAD.into()));
                EUUnload();
                std::process::exit(1);
            }
            G_P_IFACE = p_iface;

            let cert_path = state.config.eusign.sz_path.clone()
                + "EU-5E984D526F82F38F040000007383AE017103E805.cer";

            let cert = read_file_to_base64(&cert_path)?;

            let dw_error = decrypt_customer_data(&state.config, &cert, customer_data);

            if dw_error != EU_ERROR_NONE as c_ulong {
                error!("{}", get_error_message(dw_error));
                // finalize/unload
                if let Some(finalize_fn) = (*G_P_IFACE).Finalize {
                    finalize_fn();
                }
                EUUnload();
                std::process::exit(1);
            }

        // 6) Finalize the library
        let finalize_fn = (*G_P_IFACE).Finalize.unwrap();
        finalize_fn();
        EUUnload();
        }

        // 3) STORE THE DATA
    }

    Ok(Json(DiiaResponse { success: true }))
}
