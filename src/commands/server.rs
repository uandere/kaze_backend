use crate::utils::cache::{build_cache, populate_cache_from_file, CACHE_SAVE_LOCATION_DEFAULT};
use crate::utils::config::Config;
use crate::utils::db::{init_db_pool, setup_db, DbPool};
use crate::utils::diia::{diia_signature_handler, refresh_diia_session_token};
use crate::utils::eusign::*;
use crate::utils::secrets::get_secret;
use crate::utils::server_error::EUSignError;
use crate::utils::shutdown::graceful_shutdown;
use aws_config::{BehaviorVersion, Region};
use axum::routing::{delete, get, post};
use axum::Router;
use axum_server::Handle;
use clap::Parser;
use rs_firebase_admin_sdk::auth::token::cache::HttpCache;
use rs_firebase_admin_sdk::auth::token::crypto::JwtRsaPubKey;
use rs_firebase_admin_sdk::auth::token::LiveTokenVerifier;
use rs_firebase_admin_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::{c_char, CString};
use std::net::{SocketAddr, TcpListener};
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::read_to_string;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_util::task::LocalPoolHandle;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use super::utils::cache::AgreementProposalCache;
use super::utils::server_error::ServerError;

/// Database configuration from the AWS Secret
#[derive(Serialize, Deserialize)]
struct DatabaseConfig {
    username: String,
    password: String,
    #[serde(default = "default_db_host")]
    host: String,
    #[serde(default = "default_db_port")]
    port: u16,
    #[serde(default = "default_db_name")]
    dbname: String,
}

fn default_db_host() -> String {
    "kaze-rds-small.cfyeg0wimn2q.eu-central-1.rds.amazonaws.com".into()
}

fn default_db_port() -> u16 {
    5432
}

fn default_db_name() -> String {
    "diia".into()
}

/// A function that starts the server.
pub fn run(
    ServerSubcommand {
        https_port,
        config_path,
        challenge_cache_update_freq,
        agreement_template_path,
        region,
        db_secret_name,
        s3_bucket_name,
    }: ServerSubcommand,
) -> Result<(), ServerError> {
    let runtime = Runtime::new()?;

    tracing_subscriber::fmt().with_ansi(false).init();
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    runtime.block_on(async {
        let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], https_port)))?;

        let aws_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region))
            .load()
            .await;
        let aws_sm_client = aws_sdk_secretsmanager::Client::new(&aws_config);
        let aws_s3_client = aws_sdk_s3::Client::new(&aws_config);

        // Retrieve DB password from Secrets Manager
        let db_secret = get_secret(&aws_sm_client, &db_secret_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve database secret"))?;

        // Parse the JSON secret
        let db_config: DatabaseConfig = serde_json::from_str(&db_secret)
            .map_err(|e| anyhow::anyhow!("Failed to parse database secret: {}", e))?;

        // Initialize database connection
        let db_pool = init_db_pool(
            &db_config.host,
            db_config.port,
            &db_config.username,
            &db_config.password,
            &db_config.dbname,
        )
        .await?;
        setup_db(&db_pool).await?;
        info!("Database connection established successfully.");

        let config = Config::new(&config_path);

        let (signature_request_sender, mut signature_request_receiver) =
            tokio::sync::mpsc::unbounded_channel();

        // We'll keep the cache for compatibility, but gradually transition to DB
        let cache = build_cache(Arc::new(db_pool.clone()), signature_request_sender);
        populate_cache_from_file(CACHE_SAVE_LOCATION_DEFAULT, &cache).await?;

        // cache keeper task to trigger cache updates once in a while
        let cache_keeper_handle = || {
            let cache = cache.clone();
            async move {
                let mut timer = tokio::time::interval(challenge_cache_update_freq);
                loop {
                    timer.tick().await;
                    cache.run_pending_tasks().await;
                }
            }
        };
        tokio::spawn(cache_keeper_handle());

        // A code to load the EUSign library
        let encryption_cert;
        let signature_cert;
        let cert_info = ptr::null_mut();

        unsafe {
            let error_code = EULoad();
            if error_code as u32 != EU_ERROR_NONE {
                // Means it failed
                return Err(EUSignError(error_code).into());
            }

            let encryption_cert_path =
                config.eusign.sz_path.clone() + &config.eusign.encryption_cert_file_name;
            let signature_cert_path =
                config.eusign.sz_path.clone() + &config.eusign.signature_cert_file_name;

            encryption_cert = read_file_to_base64(&encryption_cert_path)?;
            signature_cert = read_file_to_base64(&signature_cert_path)?;

            // Initializing the library
            Initialize(config.clone())?;

            let c_key_path = CString::new(config.eusign.private_key_path.clone())?;
            let c_key_pwd = CString::new(config.eusign.private_key_password.clone())?;

            let error_code = EUReadPrivateKeyFile(
                c_key_path.as_ptr() as *mut c_char,
                c_key_pwd.as_ptr() as *mut c_char,
                cert_info,
            );

            if error_code as u32 != EU_ERROR_NONE {
                return Err(EUSignError(error_code).into());
            }
        }

        let agreement_template_string = Arc::new(read_to_string(agreement_template_path).await?);

        // Live Firebase App
        let gcp_service_account = credentials_provider()
            .await
            .expect("cannot receive google service account");
        let live_app = App::live(gcp_service_account)
            .await
            .expect("cannot receive google live app");
        let live_token_verifier = Arc::new(
            live_app
                .id_token_verifier()
                .await
                .expect("cannot receive google live token verifier"),
        );

        // Cache cloning is cheap, hence using state instead of an extension.
        let server_state = ServerState {
            config: Arc::new(config),
            encryption_cert: Arc::new(encryption_cert),
            signature_cert: Arc::new(signature_cert),
            cache,
            db_pool,
            agreement_template_string,
            live_token_verifier,
            aws_sm_client,
            aws_s3_client,
            s3_bucket_name,
            diia_session_token: Arc::new(Mutex::new("".into())),
        };

        // Setting up cache renewal for Diia once per every ~two hours
        let cloned_server_state = server_state.clone();
        tokio::spawn(async move {
            loop {
                let mut duration = Duration::from_secs(59 * 60 * 2); // ~two hours

                if let Err(e) = refresh_diia_session_token(cloned_server_state.clone()).await {
                    error!("wasn't able to get Diia session token: {:?}", e);
                    duration = Duration::from_secs(1); // if failed, retrying in 1 sec
                } else {
                    info!("successfully refreshed Diia session token");
                }
                sleep(duration).await;
            }
        });

        // Setting up signature request handler
        let pool = LocalPoolHandle::new(4);
        let cloned_server_state = server_state.clone();
        pool.spawn_pinned(|| async move {
            loop {
                if let Some(signature_entry) = signature_request_receiver.recv().await {
                    if let Err(e) =
                        diia_signature_handler(cloned_server_state.clone(), signature_entry).await
                    {
                        error!("couldn't handle signature reqeust: {:?}", e);
                    }
                }
            }
        });

        let cors = CorsLayer::new()
            .allow_headers(Any)
            .allow_methods(Any)
            .allow_origin(Any);

        let app = Router::new()
            .route("/", get(|| async { "Greetings from Kaze 🔑" }))
            .route(
                "/diia/signature",
                post(crate::routes::diia::signature::handler),
            )
            .route("/diia/sharing", post(crate::routes::diia::sharing::handler))
            .route(
                "/user/is_authorized",
                get(crate::routes::user::is_authorized::handler),
            )
            .route(
                "/user/get_sharing_link",
                get(crate::routes::user::get_sharing_link::handler),
            )
            .route("/user/name", get(crate::routes::user::name::handler))
            .route("/user/remove", delete(crate::routes::user::remove::handler))
            .route(
                "/agreement/generate",
                post(crate::routes::agreement::generate::handler),
            )
            .route(
                "/agreement/demo",
                post(crate::routes::agreement::demo::handler),
            )
            .route(
                "/agreement/get_sign_link",
                get(crate::routes::agreement::get_sign_link::handler),
            )
            .route(
                "/agreement/get",
                get(crate::routes::agreement::get::handler),
            )
            .route(
                "/agreement/get_signed",
                get(crate::routes::agreement::get_signed::handler),
            )
            .route(
                "/agreement/status",
                get(crate::routes::agreement::status::handler),
            )
            .route(
                "/agreement/remove",
                delete(crate::routes::agreement::remove::handler),
            )
            .layer(cors)
            .with_state(server_state.clone());

        // graceful shutdown
        let shutdown_handle = Handle::new();
        tokio::spawn(graceful_shutdown(shutdown_handle.clone(), server_state));

        info!("Starting the server...");

        axum_server::from_tcp(listener)
            .handle(shutdown_handle)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    })
}

/// A state of the server.
#[derive(Clone)]
pub struct ServerState {
    /// A config of the server
    pub config: Arc<Config>,
    /// A base64-encoded certificate for signing, matching the private keys.
    pub signature_cert: Arc<String>,
    /// A base64-encoded certificate for encryption/decryption
    pub encryption_cert: Arc<String>,
    /// A cache of agreement proposals
    pub cache: AgreementProposalCache,
    /// The database connection pool
    pub db_pool: DbPool,
    /// A string which contains a Typst template for the agreeement.
    pub agreement_template_string: Arc<String>,
    /// Firebase Token verifirer
    pub live_token_verifier:
        Arc<LiveTokenVerifier<HttpCache<reqwest::Client, BTreeMap<String, JwtRsaPubKey>>>>,
    /// AWS SM client
    pub aws_sm_client: aws_sdk_secretsmanager::Client,
    /// AWS S3 client
    pub aws_s3_client: aws_sdk_s3::Client,
    /// AWS S3 bucket name
    pub s3_bucket_name: String,
    /// Diia session token
    pub diia_session_token: Arc<Mutex<String>>,
}

#[derive(Parser, Clone)]
#[command(version = "1.0", about = "Runs the server.")]
pub struct ServerSubcommand {
    /// HTTPs port the server will listen on.
    #[arg(long, default_value_t = 3000)]
    pub https_port: u16,

    /// A path to the config file.
    #[arg(long, default_value_t = String::from("./config.toml"))]
    pub config_path: String,

    #[arg(long, default_value = "1000", value_parser = parse_duration)]
    challenge_cache_update_freq: Duration,

    /// A path to the config file.
    #[arg(long, default_value_t = String::from("./resources/typst/rental_agreement_template.typ"))]
    pub agreement_template_path: String,

    /// The region on which the AWS is running.
    #[arg(long, default_value_t = String::from("eu-central-1"))]
    region: String,

    /// The name of the secret containing database credentials
    #[arg(long, default_value_t = String::from("rds!db-8dd73543-9c21-4891-9424-1571fc376941"))]
    db_secret_name: String,

    /// The name of the s3 bucket
    #[arg(long, default_value_t = String::from("kaze-agreements"))]
    s3_bucket_name: String,
}

/// A helper function for parsing duration.
fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let milliseconds = arg.parse()?;
    Ok(Duration::from_millis(milliseconds))
}
