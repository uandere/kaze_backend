use aws_sdk_secretsmanager::Client;

pub async fn get_secret(client: &Client, secret_name: &str) -> Option<String> {
    let resp = client.get_secret_value().secret_id(secret_name).send().await.ok()?;
    Some(resp.secret_string()?.to_string())
}
