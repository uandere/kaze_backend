use super::server_error::ServerError;

// Function to upload data to S3
pub async fn upload_object(
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
    body: Vec<u8>,
    key: &str,
) -> Result<aws_sdk_s3::operation::put_object::PutObjectOutput, ServerError> {
    let body = aws_sdk_s3::primitives::ByteStream::from(body);
    client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .send()
        .await
        .map_err(ServerError::from)
}
