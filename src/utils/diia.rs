use std::{ffi::c_void, ptr::{self}, sync::Arc};

use anyhow::{anyhow, Context};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use serde::Deserialize;
use tracing::{error, info};

use super::{
    cache::AgreementProposalKey,
    db::SignatureEntry,
    eusign::{EU_CTX_SIGN_ECDSA_WITH_SHA, EU_ERROR_NONE, G_P_IFACE},
    s3::{get_agreement_pdf, upload_agreement_p7s},
    server_error::{EUSignError, ServerError},
};
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct SessionTokenResponse {
    token: String,
}

/// This function refreshes the Diia session token.
pub async fn refresh_diia_session_token(state: ServerState) -> Result<(), ServerError> {
    let client = reqwest::Client::new();

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", state.config.diia.auth_acquirer_token))?,
    );

    // Log the exact request we're making for debugging
    let url = format!(
        "{}/api/v1/auth/acquirer/{}",
        state.config.diia.host, state.config.diia.acquirer_token
    );

    // Make the GET request asynchronously
    let response = client.get(&url).headers(headers).send().await?;

    // Check the status code - important!
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        error!(
            "Failed to refresh Diia token. Status: {}, Response: {}",
            status, error_text
        );
        return Err(anyhow!("Diia API returned error status {}: {}", status, error_text).into());
    }

    // Get the response body as text
    let body: SessionTokenResponse = serde_json::from_str(&response.text().await?)?;

    // Store the raw response
    let mut lock = state.diia_session_token.lock().await;
    *lock = body.token;

    Ok(())
}

/// Adds two CAdeS signatures and stores the signed file on S3.
pub async fn diia_signature_handler(
    state: ServerState,
    SignatureEntry {
        tenant_id,
        landlord_id,
        housing_id,
        tenant_signature,
        landlord_signature,
    }: SignatureEntry,
) -> Result<(), ServerError> {
    // 1) Grab the latest PDF from S3:
    let mut pdf = get_agreement_pdf(
        &state,
        Arc::new(AgreementProposalKey {
            tenant_id: tenant_id.clone(),
            landlord_id: landlord_id.clone(),
            housing_id: housing_id.clone(),
        }),
    )
    .await?;

    // We'll be passing a pointer to EUSign's FFI:
    let pdf_data = pdf.as_mut_ptr();

    // Decode tenant signature from Base64 (must exist).
    let mut tenant_sig_bytes = BASE64_STANDARD
        .decode(&tenant_signature)
        .context("unable to decode base64 tenant signature")?;

    // If there's a landlord signature, decode it too.
    let landlord_sig_bytes = if !landlord_signature.is_empty() {
        Some(
            BASE64_STANDARD
                .decode(&landlord_signature)
                .context("unable to decode base64 landlord signature")?,
        )
    } else {
        None
    };

    // 2) Prepare function pointers from EUSign’s global interface:
    unsafe {
        let ctx_get_signer_info = (*G_P_IFACE)
            .CtxGetSignerInfo
            .context("EUSign missing CtxGetSignerInfo")?;
        let ctx_create_empty_sign = (*G_P_IFACE)
            .CtxCreateEmptySign
            .context("EUSign missing CtxCreateEmptySign")?;
        let get_signer = (*G_P_IFACE)
            .GetSigner
            .context("EUSign missing GetSigner")?;
        let get_sign_type = (*G_P_IFACE)
            .GetSignType
            .context("EUSign missing GetSignType")?;
        let ctx_append_signer = (*G_P_IFACE)
            .CtxAppendSigner
            .context("EUSign missing CtxAppendSigner")?;

        // 3) Extract the tenant’s certificate from the raw tenant signature.
        info!("---Tenant phase---");
        let mut tenant_cert_info = ptr::null_mut();
        let mut tenant_cert = ptr::null_mut();
        let mut tenant_cert_len = 0;
        let mut err = ctx_get_signer_info(
            state.ctx.lib_ctx as *mut c_void,
            0, // sign index
            tenant_sig_bytes.as_mut_ptr(),
            tenant_sig_bytes.len().try_into()?,
            &mut tenant_cert_info,
            &mut tenant_cert,
            &mut tenant_cert_len,
        );
        if err as u32 != EU_ERROR_NONE {
            return Err(EUSignError(err).into());
        }

        // 4) Create an “empty” sign container with the PDF + tenant cert.
        let mut signature = ptr::null_mut();
        let mut signature_len = 0;
        
        err = ctx_create_empty_sign(
            state.ctx.lib_ctx as *mut c_void,
            EU_CTX_SIGN_ECDSA_WITH_SHA.into(),
            pdf_data,
            pdf.len().try_into()?,
            tenant_cert,
            tenant_cert_len,
            &mut signature,
            &mut signature_len,
        );
        if err as u32 != EU_ERROR_NONE {
            return Err(EUSignError(err).into());
        }

        // 5) Convert the tenant’s raw signature blob into “signer info”:
        let mut tenant_info = ptr::null_mut();
        let mut tenant_info_len = 0;
        err = get_signer(
            0,                // sign index
            ptr::null_mut(),  // if we had a Base64 string, we’d pass it in here
            tenant_sig_bytes.as_mut_ptr(),
            tenant_sig_bytes.len().try_into()?,
            ptr::null_mut(),  // out param for base64 signer
            &mut tenant_info, // out param for raw byte-signer
            &mut tenant_info_len,
        );
        if err as u32 != EU_ERROR_NONE {
            return Err(EUSignError(err).into());
        }

        // Check the sign type (optional debugging):
        let mut tenant_sign_type = 0;
        err = get_sign_type(
            0,
            ptr::null_mut(),
            tenant_sig_bytes.as_mut_ptr(),
            tenant_sig_bytes.len().try_into()?,
            &mut tenant_sign_type,
        );
        if err as u32 != EU_ERROR_NONE {
            return Err(EUSignError(err).into());
        }
        info!("Tenant signature type code: {}", tenant_sign_type);

        // 6) Append the tenant signer onto that container:
        err = ctx_append_signer(
            state.ctx.lib_ctx as *mut c_void,
            EU_CTX_SIGN_ECDSA_WITH_SHA.into(),
            tenant_info,
            tenant_info_len,
            tenant_cert,
            tenant_cert_len,
            signature,
            signature_len,
            &mut signature,
            &mut signature_len,
        );
        if err as u32 != EU_ERROR_NONE {
            return Err(EUSignError(err).into());
        }

        // 7) If the landlord’s signature is present, do the same “append” again.
        if let Some(mut lsig) = landlord_sig_bytes {
            info!("---Landlord phase---");
            let mut landlord_cert_info = ptr::null_mut();
            let mut landlord_cert = ptr::null_mut();
            let mut landlord_cert_len = 0;
            err = ctx_get_signer_info(
                state.ctx.lib_ctx as *mut c_void,
                0,
                lsig.as_mut_ptr(),
                lsig.len().try_into()?,
                &mut landlord_cert_info,
                &mut landlord_cert,
                &mut landlord_cert_len,
            );
            if err as u32 != EU_ERROR_NONE {
                return Err(EUSignError(err).into());
            }

            let mut landlord_info = ptr::null_mut();
            let mut landlord_info_len = 0;
            err = get_signer(
                0,
                ptr::null_mut(),
                lsig.as_mut_ptr(),
                lsig.len().try_into()?,
                ptr::null_mut(),
                &mut landlord_info,
                &mut landlord_info_len,
            );
            if err as u32 != EU_ERROR_NONE {
                return Err(EUSignError(err).into());
            }

            // optional debug
            let mut landlord_sign_type = 0;
            err = get_sign_type(
                0,
                ptr::null_mut(),
                lsig.as_mut_ptr(),
                lsig.len().try_into()?,
                &mut landlord_sign_type,
            );
            if err as u32 != EU_ERROR_NONE {
                return Err(EUSignError(err).into());
            }
            info!("Landlord signature type code: {}", landlord_sign_type);

            // Append the landlord signer to the existing container:
            err = ctx_append_signer(
                state.ctx.lib_ctx as *mut c_void,
                EU_CTX_SIGN_ECDSA_WITH_SHA.into(),
                landlord_info,
                landlord_info_len,
                landlord_cert,
                landlord_cert_len,
                signature,           // pass the container that already has the tenant
                signature_len,
                &mut signature,      // updated container
                &mut signature_len,
            );
            if err as u32 != EU_ERROR_NONE {
                return Err(EUSignError(err).into());
            }
        }

        // 8) Now `signature` is a container with 1 or 2 signers. Upload it:
        upload_agreement_p7s(
            &state,
            // we must reconstruct a Vec<u8> from the raw pointer, so ownership can be moved.
            Vec::from_raw_parts(
                signature,
                signature_len.try_into()?,
                signature_len.try_into()?,
            ),
            Arc::new(AgreementProposalKey {
                tenant_id,
                landlord_id,
                housing_id,
            }),
        )
        .await?;
    }

    Ok(())
}
