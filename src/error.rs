use log::error;
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
                error!("Unauthorized from ApiError");
                return StatusCode::UNAUTHORIZED.into_response();
            }
            ApiError::Underfunded => {
                error!("Underfunded from ApiError");
                return StatusCode::PAYMENT_REQUIRED.into_response();
            }
            ApiError::Conflict => {
                error!("Conflict from ApiError");
                return StatusCode::CONFLICT.into_response();
            }
        }
    }
}

/// Turn rejections into appropriate status codes
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.find::<MissingCookie>().is_some() {
        error!("Missing Cookie Error: {:?}", err);
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if err.find::<InvalidHeader>().is_some() {
        error!("Invalid Header Error: {:?}", err);
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if let Some(custom_error) = err.find::<ApiError>() {
        match custom_error {
            ApiError::DBError(db_error) => {
                error!("DB error: {}", db_error);
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }
            ApiError::ViolatedAssertion(assertion) => {
                error!("Violated assertion: {}", assertion);
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }
            ApiError::Unauthorized => {
                error!("Unauthorized from rejection");
                return Ok(StatusCode::UNAUTHORIZED);
            }
            ApiError::Underfunded => {
                error!("Underfunded from rejection");
                return Ok(StatusCode::PAYMENT_REQUIRED);
            }
            ApiError::Conflict => {
                error!("Conflict from rejection");
                return Ok(StatusCode::CONFLICT);
            }
        }
    }

    Err(err)
}

pub async fn handle_errors<T: Reply>(
    res: Result<T, ApiError>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match res {
        Ok(r) => Ok(r.into_response()),
        Err(e) => {
            error!("{}", e);
            Ok(e.into_response())
        }
    }
}
