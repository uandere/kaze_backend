use std::{ptr::null_mut, sync::Arc};

use anyhow::{anyhow, Context};
use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use serde::Deserialize;
use tracing::error;

use super::{
    cache::AgreementProposalKey,
    eusign::{EU_ERROR_NONE, G_P_IFACE},
    s3::get_agreement_pdf,
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
    tenant_id: String,
    landlord_id: String,
) -> Result<(), ServerError> {
    let _pdf = get_agreement_pdf(
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

    let mut signature = "".to_owned();
    let mut cert_info = null_mut();
    let mut cert = null_mut();
    let cert_size = null_mut();

    unsafe {
        let ctx_get_signer_info = (*G_P_IFACE)
            .CtxGetSignerInfo
            .context("wasn't able to get ctx_get_signer_info handler")?;

        let _ctx_create_empty_sign = (*G_P_IFACE)
            .CtxCreateEmptySign
            .context("wasn't able to get ctx_create_empty_sign handler")?;

        let _get_signer = (*G_P_IFACE)
            .GetSigner
            .context("wasn't able to get get_signer handler")?;

        let _get_signs_count = (*G_P_IFACE)
            .GetSignsCount
            .context("wasn't able to get get_signs_count handler")?;

        let _get_sign_type = (*G_P_IFACE)
            .GetSignType
            .context("wasn't able to get get_sign_type handler")?;

        let _append_validators_data_to_signer_ex = (*G_P_IFACE)
            .AppendValidationDataToSignerEx
            .context("wasn't able to get get_sign_type handler")?;

        let error_code = ctx_get_signer_info(
            state.ctx.lib_ctx as *mut std::ffi::c_void,
            signature_idx,
            signature.as_mut_ptr(),
            signature.len().try_into()?,
            &mut cert_info,
            &mut cert,
            cert_size,
        );

        if error_code as u32 != EU_ERROR_NONE {
            return Err(EUSignError(error_code).into());
        }

        Ok(())
    }
}
