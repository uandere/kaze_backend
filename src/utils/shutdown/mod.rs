use axum_server::Handle;
use tracing::info;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;

/// This function is used for graceful shutdown.
/// Probably should be replaced with something more robust.
/// It was decided to panic in case we were unable to install a signal handler.
pub async fn graceful_shutdown(handle: Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C signal"),
        _ = terminate => info!("received terminate signal"),
    }

    info!("sending graceful shutdown signal");

    // TODO: HERE WE NEED TO SAVE THE SERVER'S STATE
    handle.graceful_shutdown(Some(Duration::from_secs(10)));

    loop {
        sleep(Duration::from_secs(1)).await;

        info!("alive connections: {}", handle.connection_count());
    }
}