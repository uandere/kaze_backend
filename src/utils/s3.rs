use std::sync::Arc;

use crate::commands::server::ServerState;
use anyhow::anyhow;

use super::{cache::AgreementProposalKey, server_error::ServerError};

// Uploads agreement PDF to S3
pub async fn upload_agreement_pdf(
    state: &ServerState,
    body: Vec<u8>,
    agreement_proposal_key: Arc<AgreementProposalKey>,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, ServerError> {
    let key = get_key_for_s3(agreement_proposal_key);
    let body = aws_sdk_s3::primitives::ByteStream::from(body);
    state
        .aws_s3_client
        .put_object()
        .bucket(&state.s3_bucket_name)
        .key(key)
        .body(body)
        .content_type("application/pdf") // Add this line
        .send()
        .await
        .map_err(ServerError::from)
}

// Uploads a signed agreement to S3
pub async fn upload_agreement_p7s(
    state: &ServerState,
    body: Vec<u8>,
    agreement_proposal_key: Arc<AgreementProposalKey>,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, ServerError> {
    let key = get_signature_key_for_s3(agreement_proposal_key);
    let body = aws_sdk_s3::primitives::ByteStream::from(body);
    state
        .aws_s3_client
        .put_object()
        .bucket(&state.s3_bucket_name)
        .key(key)
        .body(body)
        .content_type("application/pkcs7-signature") // Add this line
        .send()
        .await
        .map_err(ServerError::from)
}

pub fn get_key_for_s3(key: Arc<AgreementProposalKey>) -> String {
    key.tenant_id.clone() + "_" + &key.landlord_id
}

pub fn get_signature_key_for_s3(key: Arc<AgreementProposalKey>) -> String {
    "signature".to_owned() + "_" + &key.tenant_id + "_" + &key.landlord_id
}

// Returns a PDF from the S3 bucket
pub async fn get_agreement_pdf(
    state: &ServerState,
    agreement_proposal_key: Arc<AgreementProposalKey>,
) -> Result<Vec<u8>, ServerError> {
    let mut object = state
        .aws_s3_client
        .get_object()
        .bucket(state.s3_bucket_name.clone())
        .key(get_key_for_s3(agreement_proposal_key))
        .send()
        .await?;

    let mut result = vec![];

    while let Some(bytes) = object
        .body
        .try_next()
        .await
        .map_err(|err| anyhow!("Failed to read from S3 download stream: {err:?}"))?
    {
        result.append(&mut bytes.to_vec());
    }

    Ok(result)
}
