use aws_sdk_secretsmanager::Client;
use tracing::error;

pub async fn get_secret(client: &Client, secret_name: &str) -> Option<String> {
    let resp = client.get_secret_value().secret_id(secret_name).send().await;
    match resp {
        Ok(resp) => Some(resp.secret_string()?.to_string()),
        Err(e) => {
            error!("cannot retreive secret with name {}: {}", secret_name, e);
            None
        },
    }
}
