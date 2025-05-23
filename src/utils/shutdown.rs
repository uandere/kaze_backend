use axum_server::Handle;

use std::time::Duration;
use tokio::signal;
use tracing::info;

use crate::{
    commands::server::ServerState,
    utils::{
        cache::{save_cache_to_a_file, CACHE_SAVE_LOCATION_DEFAULT},
        eusign::EUUnload,
    },
};

/// This function is used for graceful shutdown.
/// Probably should be replaced with something more robust.
/// It was decided to panic in case we were unable to install a signal handler.
pub async fn graceful_shutdown(handle: Handle, state: ServerState) {
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

    // Saving the cache into the file for now
    save_cache_to_a_file(CACHE_SAVE_LOCATION_DEFAULT, state.cache).await;

    handle.graceful_shutdown(Some(Duration::from_secs(10)));

    // Free the EUSign library
    unsafe {
        EUUnload();
    }
}
