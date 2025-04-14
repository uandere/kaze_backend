use std::{ffi::c_int, fs};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct EUSignConfig {
    pub private_key_path: String,
    pub private_key_password: String,
    pub cas_json_path: String,
    pub ca_certificates_path: String,
    pub sz_path: String,
    pub proxy_use: c_int,
    pub proxy_address: String,
    pub proxy_port: String,
    pub proxy_user: String,
    pub proxy_password: String,
    pub default_ocsp_server: String,
    pub default_tsp_server: String,
    pub encryption_cert_file_name: String,
    pub signature_cert_file_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DiiaConfig {
    pub acquirer_token: String,
    pub auth_acquirer_token: String,
    pub host: String,
    pub branch_id: String,
    pub offer_sharing_id: String,
    pub offer_signing_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub eusign: EUSignConfig,
    pub diia: DiiaConfig,
}

impl Config {
    pub fn new(path: &str) -> Self {
        let config_file_content = fs::read_to_string(path).unwrap_or_else(|e| {
            panic!("unable to read the config file at path: {path}, error: {e}")
        });

        toml::from_str(&config_file_content)
            .unwrap_or_else(|e| panic!("cannot parse config file: {e}"))
    }
}
