use std::{ptr::null_mut, sync::Arc};

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
        ..
    }: SignatureEntry,
) -> Result<(), ServerError> {
    let mut pdf = get_agreement_pdf(
        &state,
        Arc::new(AgreementProposalKey {
            tenant_id: tenant_id.clone(),
            landlord_id: landlord_id.clone(),
            housing_id: housing_id.clone(),
        }),
    )
    .await?;

    let pdf_data = pdf.as_mut_ptr();

    let mut tenant_signature = BASE64_STANDARD.decode(&tenant_signature)?;

    unsafe {
        let ctx_get_signer_info = (*G_P_IFACE)
            .CtxGetSignerInfo
            .context("wasn't able to get ctx_get_signer_info handler")?;

        let ctx_create_empty_sign = (*G_P_IFACE)
            .CtxCreateEmptySign
            .context("wasn't able to get ctx_create_empty_sign handler")?;

        let get_signer = (*G_P_IFACE)
            .GetSigner
            .context("wasn't able to get get_signer handler")?;

        let _ctx_get_signs_count = (*G_P_IFACE)
            .CtxGetSignsCount
            .context("wasn't able to get get_signs_count handler")?;

        let get_sign_type = (*G_P_IFACE)
            .GetSignType
            .context("wasn't able to get get_sign_type handler")?;

        let _append_validators_data_to_signer_ex = (*G_P_IFACE)
            .AppendValidationDataToSignerEx
            .context("wasn't able to get append_validators_data_to_signer_ex handler")?;

        let ctx_append_signer = (*G_P_IFACE)
            .CtxAppendSigner
            .context("wasn't able to get ctx_append_signer handler")?;

        // building a CAdeS signature
        let mut signature = null_mut();
        let mut signature_len = 0;

        let error_code = ctx_create_empty_sign(
            state.ctx.lib_ctx as *mut std::ffi::c_void,
            EU_CTX_SIGN_ECDSA_WITH_SHA.into(),
            pdf_data,
            pdf.len().try_into()?,
            (*(state.cert)).clone().as_mut_ptr(),
            state.cert.len().try_into()?,
            &mut signature,
            &mut signature_len,
        );

        info!("Here1");

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        let mut tenant_cert_info = null_mut();
        let mut tenant_cert = null_mut();
        let mut tenant_cert_len = 0;

        let error_code = ctx_get_signer_info(
            state.ctx.lib_ctx as *mut std::ffi::c_void,
            0,
            tenant_signature.as_mut_ptr(),
            tenant_signature.len().try_into()?,
            &mut tenant_cert_info,
            &mut tenant_cert,
            &mut tenant_cert_len,
        );

        info!("Here2");

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        let mut tenant_info = null_mut();
        let mut tenant_info_len = 0;

        let error_code = get_signer(
            0,
            null_mut(),
            tenant_signature.as_mut_ptr(),
            tenant_signature.len().try_into()?,
            null_mut(),
            &mut tenant_info,
            &mut tenant_info_len,
        );

        info!("Here3");

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        let mut signature_type = 0;

        let error_code = get_sign_type(
            0,
            null_mut(),
            tenant_signature.as_mut_ptr(),
            tenant_signature.len().try_into()?,
            &mut signature_type,
        );

        info!("Here4");

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        let error_code = ctx_append_signer(
            state.ctx.lib_ctx as *mut std::ffi::c_void,
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

        info!("Here5");

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        upload_agreement_p7s(
            &state,
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
        ).await?;
    }

    Ok(())
}
