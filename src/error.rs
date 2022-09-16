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

    #[error("NotFound")]
    NotFound,

    #[error("Underfunded")]
    Underfunded,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("{0}")]
    ViolatedAssertion(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::DBError(error) => {
                error!("{:?}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Conflict => StatusCode::CONFLICT,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Underfunded => StatusCode::PAYMENT_REQUIRED,
            AppError::ViolatedAssertion(assertion) => {
                error!("{}", assertion);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        status.into_response()
    }
}
