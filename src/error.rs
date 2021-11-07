use log::{error};
use thiserror::Error;
use warp::http::StatusCode;
use warp::reject::{InvalidHeader, MissingCookie};
use warp::reply::Response;
use warp::{Rejection, Reply};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error")]
    DBError(#[from] sqlx::Error),

    #[error("{0}")]
    ViolatedAssertion(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Underfunded")]
    Underfunded,

    #[error("Conflict")]
    Conflict,
}

impl warp::reject::Reject for ApiError {}

impl warp::reply::Reply for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DBError(db_error) => {
                error!("DB error: {}", db_error);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            ApiError::ViolatedAssertion(assertion) => {
                error!("Violated assertion: {}", assertion);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            ApiError::Unauthorized => {
                return StatusCode::UNAUTHORIZED.into_response();
            }
            ApiError::Underfunded => {
                return StatusCode::PAYMENT_REQUIRED.into_response();
            }
            ApiError::Conflict => {
                return StatusCode::CONFLICT.into_response();
            }
        }
    }
}

/// Turn rejections into appropriate status codes
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.find::<MissingCookie>().is_some() {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if err.find::<InvalidHeader>().is_some() {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if let Some(custom_error) = err.find::<ApiError>() {
        match custom_error {
            ApiError::DBError(_db_error) => {
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }
            ApiError::ViolatedAssertion(_assertion) => {
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }
            ApiError::Unauthorized => {
                return Ok(StatusCode::UNAUTHORIZED);
            }
            ApiError::Underfunded => {
                return Ok(StatusCode::PAYMENT_REQUIRED);
            }
            ApiError::Conflict => {
                return Ok(StatusCode::CONFLICT);
            }
        }
    }

    Err(err)
}

pub async fn handle_errors<T: Reply, E: Reply>(
    res: Result<T, E>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match res {
        Ok(r) => Ok(r.into_response()),
        Err(e) => Ok(e.into_response()),
    }
}
