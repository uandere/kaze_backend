use std::ffi::{c_char, c_ulong, c_void, CString};
use std::net::{SocketAddr, TcpListener};
use std::ptr;
use std::sync::Arc;
use std::time::Duration;
// use aws_config::meta::region::RegionProviderChain;
// use aws_sdk_secretsmanager::config::Region;
use crate::routes::request_id::diia_user_info;
use crate::utils::config::Config;
use crate::utils::eusign::*;
use crate::utils::server_error::EusignError;
use crate::utils::shutdown::graceful_shutdown;
use axum::routing::{get, post};
use axum::Router;
use axum_server::Handle;
use clap::Parser;
use http::Method;
use tokio::runtime::Runtime;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use super::utils::server_error::ServerError;

/// A function that starts the server.
/// For now, we can configure the port to run on.
pub fn run(
    ServerSubcommand {
        https_port,
        config_path,
        // region,
    }: ServerSubcommand,
) -> Result<(), ServerError> {
    let runtime = Runtime::new()?;

    tracing_subscriber::fmt().with_ansi(false).init();

    runtime.block_on(async {
        let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], https_port)))?;

        // let region_provider = RegionProviderChain::first_try(Region::new(region))
        //     .or_default_provider()
        //     .or_else(Region::new("eu-central-1"));

        // let aws_config = aws_config::from_env().region(region_provider).load().await;
        // let aws_sm_client = aws_sdk_secretsmanager::Client::new(&aws_config);

        let config = Config::new(&config_path);

        // A code to load the EUSign library
        let cert;
        let lib_ctx: *mut c_void;
        let mut key_ctx: *mut c_void = ptr::null_mut();

        unsafe {
            let loaded = EULoad();
            if loaded == 0 {
                // Means it failed
                error!("{}", get_error_message(EU_ERROR_LIBRARY_LOAD.into()));
                return Err(EusignError(EU_ERROR_LIBRARY_LOAD as c_ulong).into());
            }

            // 2) Get the interface pointer
            let p_iface = EUGetInterface();
            if p_iface.is_null() {
                error!("{}", get_error_message(EU_ERROR_LIBRARY_LOAD.into()));
                EUUnload();
                return Err(EusignError(EU_ERROR_LIBRARY_LOAD as c_ulong).into());
            }
            G_P_IFACE = p_iface;

            let cert_path = config.eusign.sz_path.clone()
                + "EU-5E984D526F82F38F040000007383AE017103E805.cer";

            cert = read_file_to_base64(&cert_path)?;

            // Creating library context
            lib_ctx = Initialize(config.clone())?;

            let ctx_read_private_key_file = (*G_P_IFACE).CtxReadPrivateKeyFile.unwrap();

            let c_key_path = CString::new(config.eusign.private_key_path.clone())?;
            let c_key_pwd = CString::new(config.eusign.private_key_password.clone())?;

            ctx_read_private_key_file(lib_ctx, c_key_path.as_ptr() as *mut c_char, c_key_pwd.as_ptr() as *mut c_char, &mut key_ctx, ptr::null_mut());
            
        }

        // Cache cloning is cheap, hence using state instead of an extension.
        let server_state = ServerState {
            config: Arc::new(config),
            cert: Arc::new(cert),
            ctx: Arc::new(EusignContext { lib_ctx, key_ctx }),
            // aws_sm_client
        };

        let cors = CorsLayer::new()
            .allow_headers(Any)
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any);

        // TODO: ask Diia team to change diia_sharing route to diia/sharing and
        // diia_signature to diia/signature.
        let app = Router::new()
            .route("/", get(|| async { "Greetings from Kaze 🔑" }))
            .route(
                "/diia/signature",
                get(crate::routes::diia_signature::diia_signature),
            )
            .route(
                "/diia/sharing",
                post(crate::routes::diia_sharing::diia_sharing),
            )
            .route("/diia/is_user_authorized", get(diia_user_info))
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
    pub config: Arc<Config>,
    pub cert: Arc<String>,
    /// A pointer to the context of the library
    pub ctx: Arc<EusignContext>,
    // aws_sm_client: aws_sdk_secretsmanager::Client
}

#[derive(Parser, Clone)]
#[command(version = "1.0", about = "Runs the server.")]
pub struct ServerSubcommand {
    /// HTTPs port the server will listen on.
    #[arg(long, default_value_t = 3000)]
    https_port: u16,

    #[arg(long, default_value_t = String::from("./config.toml"))]
    config_path: String,
    // /// The region on which the AWS is running.
    // #[arg(long, default_value_t = String::from("eu-central-1"))]
    // region: String
}

/// A helper function for parsing duration.
fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let milliseconds = arg.parse()?;
    Ok(Duration::from_millis(milliseconds))
}
