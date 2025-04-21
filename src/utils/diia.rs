use std::{
    ffi::c_ulong,
    ptr::{self, null_mut},
    sync::Arc,
};

use anyhow::{anyhow, Context};
use base64::{prelude::BASE64_STANDARD, Engine as _};
use http::{
    header::{ACCEPT, AUTHORIZATION},
    HeaderMap, HeaderValue,
};
use serde::Deserialize;
use tracing::info;

use super::{
    cache::AgreementProposalKey,
    db::SignatureEntry,
    eusign::{EU_ERROR_NONE, G_P_IFACE},
    s3::{get_agreement_pdf, upload_agreement_p7s},
    server_error::{EUSignError, ServerError},
};
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct SessionTokenResponse {
    pub token: String,
}

/// This function refreshes the Diia session token.
pub async fn refresh_diia_session_token(state: ServerState) -> Result<(), ServerError> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", state.config.diia.auth_acquirer_token))?,
    );

    let url = format!(
        "{}/api/v1/auth/acquirer/{}",
        state.config.diia.host, state.config.diia.acquirer_token
    );

    let response = client.get(&url).headers(headers).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!("Diia API returned error status {}: {}", status, error_text).into());
    }

    let body: SessionTokenResponse = serde_json::from_str(&response.text().await?)?;

    let mut lock = state.diia_session_token.lock().await;
    *lock = body.token;

    Ok(())
}

/// Adds two CAdES signatures and stores the signed file on S3.
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
    // 1) fetch the PDF
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

    // 2) decode both Base64 blobs
    let mut tenant_sig_bytes = BASE64_STANDARD
        .decode(&tenant_signature)
        .context("unable to decode tenant signature")?;
    let mut landlord_sig_bytes = BASE64_STANDARD
        .decode(&landlord_signature)
        .context("unable to decode landlord signature")?;

    unsafe {
        // 3) grab all the EUSign entry points
        let get_signer_info = (*G_P_IFACE).GetSignerInfo.unwrap();
        let create_empty_sign = (*G_P_IFACE).CreateEmptySign.unwrap();
        let get_signer = (*G_P_IFACE).GetSigner.unwrap();
        let get_sign_type = (*G_P_IFACE).GetSignType.unwrap();
        let append_signer = (*G_P_IFACE).AppendSigner.unwrap();

        // frees raw buffers
        let free_memory = (*G_P_IFACE).FreeMemory.unwrap();
        // frees the EU_CERT_INFO_EX struct
        let free_cert_ex = (*G_P_IFACE).FreeCertificateInfoEx.unwrap();

        // ---- Tenant phase ----

        // a) pull out certificate‐info + raw certificate
        let mut tenant_cert_info = ptr::null_mut();
        let mut tenant_cert = ptr::null_mut();
        let mut tenant_cert_len = 0;
        let err = get_signer_info(
            0,
            null_mut(),
            tenant_sig_bytes.as_mut_ptr(),
            tenant_sig_bytes.len().try_into()?,
            &mut tenant_cert_info,
            &mut tenant_cert,
            &mut tenant_cert_len,
        );
        if err != EU_ERROR_NONE as c_ulong {
            return Err(EUSignError(err).into());
        }

        // b) extract the raw signer‐info blob
        let mut tenant_info = ptr::null_mut();
        let mut tenant_info_len = 0;
        let err = get_signer(
            0,
            ptr::null_mut(),
            tenant_sig_bytes.as_mut_ptr(),
            tenant_sig_bytes.len().try_into()?,
            ptr::null_mut(),
            &mut tenant_info,
            &mut tenant_info_len,
        );
        if err != EU_ERROR_NONE as c_ulong {
            // cleanup the cert we just got
            free_cert_ex(tenant_cert_info);
            free_memory(tenant_cert);
            return Err(EUSignError(err).into());
        }

        // optional: log sign‐type
        {
            let mut ttype = 0;
            let _ = get_sign_type(
                0,
                ptr::null_mut(),
                tenant_sig_bytes.as_mut_ptr(),
                tenant_sig_bytes.len().try_into()?,
                &mut ttype,
            );
            info!("Tenant signature type: {}", ttype);
        }

        // c) make an “empty” CAdES container
        let mut signature0 = ptr::null_mut();
        let mut signature0_len = 0;
        let err = create_empty_sign(
            pdf_data,
            pdf.len().try_into()?,
            // EU_CTX_SIGN_ECDSA_WITH_SHA.into(),
            // tenant_cert,
            // tenant_cert_len,
            null_mut(),
            &mut signature0,
            &mut signature0_len,
        );
        if err != EU_ERROR_NONE as c_ulong {
            // cleanup everything from tenant phase
            free_memory(tenant_info);
            free_cert_ex(tenant_cert_info);
            free_memory(tenant_cert);
            return Err(EUSignError(err).into());
        }

        // d) append the tenant signer
        let mut signature1 = ptr::null_mut();
        let mut signature1_len = 0;
        let err = append_signer(
            null_mut(),
            tenant_info,
            tenant_info_len,
            tenant_cert,
            tenant_cert_len,
            null_mut(),
            signature0,
            signature0_len,
            null_mut(),
            &mut signature1,
            &mut signature1_len,
        );
        if err != EU_ERROR_NONE as c_ulong {
            // cleanup the empty container + tenant info
            free_memory(signature0);
            free_memory(tenant_info);
            free_cert_ex(tenant_cert_info);
            free_memory(tenant_cert);
            return Err(EUSignError(err).into());
        }

        // free intermediate buffers from tenant phase
        free_memory(signature0);
        free_memory(tenant_info);
        free_cert_ex(tenant_cert_info);
        free_memory(tenant_cert);

        // ---- Landlord phase (optional) ----
        let (final_ptr, final_len) = if !landlord_sig_bytes.is_empty() {
            info!("--- Landlord phase ---");

            // a) get landlord's cert + info
            let mut landlord_cert_info = ptr::null_mut();
            let mut landlord_cert = ptr::null_mut();
            let mut landlord_cert_len = 0;
            let err = get_signer_info(
                0,
                null_mut(),
                landlord_sig_bytes.as_mut_ptr(),
                landlord_sig_bytes.len().try_into()?,
                &mut landlord_cert_info,
                &mut landlord_cert,
                &mut landlord_cert_len,
            );
            if err != EU_ERROR_NONE as c_ulong {
                free_memory(signature1);
                return Err(EUSignError(err).into());
            }

            // b) pull out raw signer info
            let mut landlord_info = ptr::null_mut();
            let mut landlord_info_len = 0;
            let err = get_signer(
                0,
                ptr::null_mut(),
                landlord_sig_bytes.as_mut_ptr(),
                landlord_sig_bytes.len().try_into()?,
                ptr::null_mut(),
                &mut landlord_info,
                &mut landlord_info_len,
            );
            if err != EU_ERROR_NONE as c_ulong {
                free_cert_ex(landlord_cert_info);
                free_memory(landlord_cert);
                free_memory(signature1);
                return Err(EUSignError(err).into());
            }

            // optional debug landlord sign type
            {
                let mut ltype = 0;
                let _ = get_sign_type(
                    0,
                    ptr::null_mut(),
                    landlord_sig_bytes.as_mut_ptr(),
                    landlord_sig_bytes.len().try_into()?,
                    &mut ltype,
                );
                info!("Landlord signature type: {}", ltype);
            }

            // c) append landlord onto the existing container
            let mut signature2 = ptr::null_mut();
            let mut signature2_len = 0;
            let err = append_signer(
                null_mut(),
                landlord_info,
                landlord_info_len,
                landlord_cert,
                landlord_cert_len,
                null_mut(),
                signature1,
                signature1_len,
                null_mut(),
                &mut signature2,
                &mut signature2_len,
            );
            if err != EU_ERROR_NONE as c_ulong {
                free_memory(signature1);
                free_memory(landlord_info);
                free_cert_ex(landlord_cert_info);
                free_memory(landlord_cert);
                return Err(EUSignError(err).into());
            }

            // free the old container + landlord metadata
            free_memory(signature1);
            free_memory(landlord_info);
            free_cert_ex(landlord_cert_info);
            free_memory(landlord_cert);

            (signature2, signature2_len)
        } else {
            // no landlord sig → this is final
            (signature1, signature1_len)
        };

        // 4) move it into a Rust Vec<u8>
        let out = std::slice::from_raw_parts(final_ptr, final_len as usize).to_vec();

        // 5) free the last C++ buffer
        free_memory(final_ptr);

        // 6) upload
        upload_agreement_p7s(
            &state,
            out,
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
