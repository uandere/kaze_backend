use std::net::{SocketAddr, TcpListener};
use std::time::Duration;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::config::Region;
use axum::routing::get;
use axum::Router;
use axum_server::Handle;
use clap::Parser;
use http::Method;
use tokio::runtime::Runtime;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use crate::utils::shutdown::graceful_shutdown;

/// A function that starts the server.
/// For now, we can configure the port to run on.
pub fn run(
    ServerSubcommand {
        https_port,
        region,
    }: ServerSubcommand,
) {
    let runtime = Runtime::new().unwrap();

    tracing_subscriber::fmt()
        .with_ansi(false)
        .init();


    runtime.block_on(async {
        let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], https_port))).unwrap();

        let region_provider = RegionProviderChain::first_try(Region::new(region))
            .or_default_provider()
            .or_else(Region::new("eu-central-1"));

        let aws_config = aws_config::from_env().region(region_provider).load().await;
        let aws_sm_client = aws_sdk_secretsmanager::Client::new(&aws_config);

        // Cache cloning is cheap, hence using state instead of an extension.
        let server_state = ServerState {
            aws_sm_client
        };


        let cors = CorsLayer::new()
            .allow_headers(Any)
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any);

        let app = Router::new()
            .route("/", get(|| async { "Greetings from Kaze 🔑" }))
            .route("/documents/generate", get(crate::routes::generate::generate))
            .route("/diia", get(crate::routes::diia::diia))
            .layer(cors)
            .with_state(server_state);

        // graceful shutdown
        let shutdown_handle = Handle::new();
        tokio::spawn(graceful_shutdown(shutdown_handle.clone()));

        info!("Starting the server...");

        axum_server::from_tcp(listener)
            .handle(shutdown_handle)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

/// A state of the server.
#[derive(Clone)]
pub struct ServerState {
    aws_sm_client: aws_sdk_secretsmanager::Client
}

#[derive(Parser, Clone)]
#[command(version = "1.0", about = "Runs the server.")]
pub struct ServerSubcommand {
    /// HTTPs port the server will listen on.
    #[arg(long, default_value_t = 3000)]
    https_port: u16,
    /// The region on which the AWS is running.
    #[arg(long, default_value_t = String::from("eu-central-1"))]
    region: String
}


/// A helper function for parsing duration.
fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    let milliseconds = arg.parse()?;
    Ok(Duration::from_millis(milliseconds))
}