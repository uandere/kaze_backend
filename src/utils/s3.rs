use crate::commands::server::ServerState;
use anyhow::anyhow;
use uuid::Uuid;

use super::server_error::ServerError;

// Uploads agreement PDF to S3
pub async fn upload_agreement_pdf(
    state: &ServerState,
    body: Vec<u8>,
    tenant_id: Uuid,
    landlord_id: Uuid,
    housing_id: Uuid,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, ServerError> {
    let key = get_key_for_s3(tenant_id, landlord_id, housing_id);
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
    tenant_id: Uuid,
    landlord_id: Uuid,
    housing_id: Uuid,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, ServerError> {
    let key = get_signature_key_for_s3(tenant_id, landlord_id, housing_id);
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

pub fn get_key_for_s3(tenant_id: Uuid, landlord_id: Uuid, housing_id: Uuid) -> String {
    tenant_id.to_string() + "_" + &landlord_id.to_string() + "_" + &housing_id.to_string()
}

pub fn get_signature_key_for_s3(tenant_id: Uuid, landlord_id: Uuid, housing_id: Uuid) -> String {
    tenant_id.to_string()
        + "_"
        + &landlord_id.to_string()
        + "_"
        + &housing_id.to_string()
        + "_signed"
}

// Returns a PDF from the S3 bucket
pub async fn get_agreement_pdf(
    state: &ServerState,
    tenant_id: Uuid,
    landlord_id: Uuid,
    housing_id: Uuid,
) -> Result<Vec<u8>, ServerError> {
    let mut object = state
        .aws_s3_client
        .get_object()
        .bucket(state.s3_bucket_name.clone())
        .key(get_key_for_s3(tenant_id, landlord_id, housing_id))
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

// Returns a signed PDF from the S3 bucket.
pub async fn get_agreement_ps7(
    state: &ServerState,
    tenant_id: Uuid,
    landlord_id: Uuid,
    housing_id: Uuid,
) -> Result<Vec<u8>, ServerError> {
    let mut object = state
        .aws_s3_client
        .get_object()
        .bucket(state.s3_bucket_name.clone())
        .key(get_signature_key_for_s3(tenant_id, landlord_id, housing_id))
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
