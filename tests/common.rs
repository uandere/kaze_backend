#![allow(async_fn_in_trait)]
use anyhow::anyhow;
use std::fs;

use anyhow::Result;
use chrono::Duration;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderMap, HeaderValue,
};
use kaze_backend::{
    routes::{
        agreement::get_sign_link::SignHashRequestId, user::get_sharing_link::DiiaSharingRequestId,
    },
    utils::{config::Config, diia::SessionTokenResponse},
};
use uuid::Uuid;

pub trait Request {
    async fn send(&self, endpoint: &str) -> Result<Duration>;
}

#[derive(Default)]
pub struct Field {
    pub name: String,
    pub file_name: String,
    pub content: Vec<u8>,
}

#[derive(Default)]
pub struct SharingRequestContent {
    pub internal_passport: Field,
    pub taxpayer_card: Field,
    pub encode_data: Field,
}

#[derive(Default)]
pub struct SharingRequest {
    pub headers: HeaderMap,
    pub content: SharingRequestContent,
}

#[derive(Default)]
pub struct SigningRequestContent {
    encode_data: Field,
}

#[derive(Default)]
pub struct SigningRequest {
    pub headers: HeaderMap,
    pub content: SigningRequestContent,
}

#[derive(Default)]
pub struct GenerateRequestContent {
    pub tenant_id: String,
    pub landlord_id: String,
    pub housing_id: String,
    pub _uid: String,
}

#[derive(Default)]
pub struct GenerateRequest {
    pub headers: HeaderMap,
    pub content: GenerateRequestContent,
}

#[derive(Default)]
pub struct Setup {
    pub sharing_requests: Vec<SharingRequest>,
    pub generate_requests: Vec<GenerateRequest>,
    pub signing_requests: Vec<SigningRequest>,
}

pub async fn get_diia_session_token(config: Config) -> Result<String> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {}", config.diia.auth_acquirer_token))?,
    );

    let url = format!(
        "{}/api/v1/auth/acquirer/{}",
        config.diia.host, config.diia.acquirer_token
    );

    let response = client.get(&url).headers(headers).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;
        return Err(anyhow!(
            "Diia API returned error status {}: {}",
            status,
            error_text
        ));
    }

    let body: SessionTokenResponse = serde_json::from_str(&response.text().await?)?;

    Ok(body.token)
}

pub async fn setup(path_to_signature: &str, users_dir: &str) -> Result<Setup> {
    let config = Config::new("./config.toml");
    let diia_session_token = get_diia_session_token(config).await?;

    let mut setup = Setup::default();

    // constructing sharing requests
    for i in 1..5 {
        let mut sharing_request = SharingRequest::default();
        for entry in fs::read_dir(format!("{users_dir}/landlords/{i}"))? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("passport") {
                    sharing_request.content.internal_passport.content = fs::read(path.clone())?;
                    sharing_request.content.internal_passport.file_name = name.into();
                    sharing_request.content.internal_passport.name = "internal-passport".into();
                } else if name.contains("card") {
                    sharing_request.content.taxpayer_card.content = fs::read(path.clone())?;
                    sharing_request.content.taxpayer_card.file_name = name.into();
                    sharing_request.content.taxpayer_card.name = "taxpayer-card".into();
                } else {
                    sharing_request.content.encode_data.content = fs::read(path.clone())?;
                    sharing_request.content.encode_data.file_name = name.into();
                    sharing_request.content.encode_data.name = "encodeData".into();
                }
            }
        }

        sharing_request.headers.append(
            CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data"),
        );
        sharing_request.headers.append(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {diia_session_token}"))?,
        );
        sharing_request
            .headers
            .append(ACCEPT, HeaderValue::from_static("application/json"));
        sharing_request.headers.append(
            "X-Document-Request-Trace-Id",
            HeaderValue::from_str(&serde_json::to_string(&DiiaSharingRequestId {
                uid: format!("landlord{i}"),
                seed: Uuid::new_v4(),
            })?)?,
        );

        setup.sharing_requests.push(sharing_request);
    }

    // constructing generating requests
    for i in 1..5 {
        let mut generate_request = GenerateRequest {
            headers: HeaderMap::new(),
            content: GenerateRequestContent {
                tenant_id: format!("landlord{i}"),
                landlord_id: format!("landlord{i}"),
                housing_id: "housing1".into(),
                _uid: format!("landlord{i}"),
            },
        };

        generate_request
            .headers
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        setup.generate_requests.push(generate_request);
    }

    // constructing signing requests
    let encode_data = fs::read(path_to_signature)?;
    for i in 1..5 {
        let mut signing_request = SigningRequest::default();
        signing_request.content.encode_data.content = encode_data.clone();
        signing_request.content.encode_data.file_name = "encodeData".to_string();
        signing_request.content.encode_data.name = "encodeData".to_string();

        signing_request.headers.append(
            CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data"),
        );

        signing_request.headers.append(
            "X-Document-Request-Trace-Id",
            HeaderValue::from_str(&serde_json::to_string(&SignHashRequestId {
                seed: Uuid::new_v4(),
                tenant_id: format!("landlord{i}"),
                landlord_id: format!("landlord{i}"),
                signed_by: format!("landlord{i}"),
                housing_id: "housing1".into(),
            })?)?,
        );

        signing_request
            .headers
            .append("X-Diia-Id-Action", HeaderValue::from_static("auth"));

        setup.signing_requests.push(signing_request);
    }

    Ok(setup)
}
