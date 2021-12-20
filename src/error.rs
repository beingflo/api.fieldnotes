use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use log::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
    DBError(#[from] sqlx::Error),

    #[error("Conflict")]
    Conflict,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("{0}")]
    ViolatedAssertion(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::DBError(error) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Conflict => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::ViolatedAssertion(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        status.into_response()
    }
}
