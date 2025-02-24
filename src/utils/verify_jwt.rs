use anyhow::anyhow;
use rs_firebase_admin_sdk::auth::token::TokenVerifier;
use tracing::*;

use crate::commands::server::ServerState;

use super::server_error::ServerError;

async fn verify_token<T: TokenVerifier>(token: &str, verifier: &T) -> Result<String, ServerError> {
    match verifier.verify_token(token).await {
        Ok(token) => {
            let user_id = token.critical_claims.sub;
            info!("Token for user {user_id} is valid!");
            Ok(user_id)
        }
        Err(err) => {
            warn!("Token is invalid because {err}!");
            Err(anyhow!("Your token is incorrect!").into())
        }
    }
}


pub async fn verify_jwt(token: &str, state: &ServerState) -> Result<String, ServerError> {
    verify_token(token, state.live_token_verifier.as_ref()).await?
}