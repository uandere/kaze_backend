use std::ffi::c_ulong;

use anyhow::anyhow;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

use super::eusign::get_error_message;

#[derive(Debug)]
pub struct ServerError(pub anyhow::Error);

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Server error: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`.
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub struct EusignError(pub c_ulong);

impl From<EusignError> for ServerError {
    fn from(err: EusignError) -> Self {
        Self(anyhow!(format!("Eusign error: {}", unsafe {
            get_error_message(err.0)
        })))
    }
}
