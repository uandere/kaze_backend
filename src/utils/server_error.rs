//! Centralised error‑to‑HTTP mapping for every Axum handler.
use std::ffi::c_ulong;

use anyhow::Error;
use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::Serialize;
use tracing::error;

/// *Public* payload returned to the client on every error.
#[derive(Serialize)]
struct ErrorResponse {
    /// Stable machine‑readable identifier (snake‑case).
    code: &'static str,
    /// Human‑readable description that is safe to expose.
    message: String,
}

/// Top‑level error type used throughout the code‑base.
///
/// Anything that implements `Into<anyhow::Error>` can be converted into
/// `ServerError` via the blanket `From` implementation at the bottom – this is
/// what enables the ubiquitous `?` operator.
#[derive(Debug)]
pub enum ServerError {
    /// 400 – the caller provided invalid data.
    BadRequest(String),
    /// 401 – the caller is not authenticated or the token is invalid/expired.
    Unauthorized(String),
    /// 404 – the requested resource does not exist (or does not belong to the caller).
    NotFound(String),
    /// 409 – business‑logic conflict (duplicate, already exists, etc.).
    Conflict(String),
    /// Special wrapper for errors coming from the EUSignCP FFI.
    Eusign(EUSignError),
    /// 500 – any other error that we did not explicitly classify.
    Internal(Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServerError::BadRequest(msg) => json(StatusCode::BAD_REQUEST, "bad_request", msg),
            ServerError::Unauthorized(msg) => {
                json(StatusCode::UNAUTHORIZED, "unauthorized", msg)
            }
            ServerError::NotFound(msg) => json(StatusCode::NOT_FOUND, "not_found", msg),
            ServerError::Conflict(msg) => json(StatusCode::CONFLICT, "conflict", msg),
            ServerError::Eusign(err) => {
                // We keep only a terse public message. Full diagnostics go to the log.
                error!(code = err.0, msg = %err.internal_message(), "EUSign error");
                json(
                    StatusCode::BAD_REQUEST,
                    "eusign_error",
                    err.public_message(),
                )
            }
            ServerError::Internal(err) => {
                // Log the full chain for debugging.
                error!(error = %err, "Unhandled internal error: ");
                json(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Internal server error. Please try again later.".to_string(),
                )
            }
        }
    }
}

/// Helper – build a `(StatusCode, Json<ErrorResponse>)` and convert to `Response`.
fn json(code: StatusCode, tag: &'static str, message: String) -> axum::response::Response {
    (code, Json(ErrorResponse { code: tag, message })).into_response()
}

//──────────────────────────────────────────────────────────────────────────────
//  EUSign error wrapper
//──────────────────────────────────────────────────────────────────────────────

/// New‑type so we can impl `From<EUSignError>` separately without clashing with
/// the blanket `From<anyhow::Error>` below.
#[derive(Debug, Clone, Copy)]
pub struct EUSignError(pub c_ulong);

impl EUSignError {
    /// Internal – full text from the native library (unsafe, logs only).
    pub fn internal_message(self) -> String {
        // Safe because the function returns a valid C string (library contract).
        unsafe { crate::utils::eusign::get_error_message(self.0) }
    }

    /// Public – a short, non‑revealing description sent to the client.
    pub fn public_message(self) -> String {
        // We intentionally do *not* propagate the full error string to avoid
        // leaking library or certificate details.
        format!("EUSign operation failed (code {})", self.0)
    }
}

impl From<EUSignError> for ServerError {
    fn from(err: EUSignError) -> Self {
        ServerError::Eusign(err)
    }
}

//──────────────────────────────────────────────────────────────────────────────
//  Blanket conversions
//──────────────────────────────────────────────────────────────────────────────

/// Anything that can be turned into `anyhow::Error` becomes an *internal* error
/// unless the caller opted into a more specific variant (`BadRequest`, etc.).
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        ServerError::Internal(err.into())
    }
}
