

use axum::extract::State;

use crate::commands::server::ServerState;

use super::server_error::ServerError;

// Function to upload data to S3
pub async fn upload_object(
    state: &State<ServerState>,
    body: Vec<u8>,
    key: &str,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, ServerError> {
    let body = aws_sdk_s3::primitives::ByteStream::from(body);
    state.aws_s3_client
        .put_object()
        .bucket(&state.s3_bucket_name)
        .key(key)
        .body(body)
        .send()
        .await
        .map_err(ServerError::from)
}
