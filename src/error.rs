use log::error;
use thiserror::Error;
use warp::http::StatusCode;
use warp::{Rejection, Reply};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error")]
    DBError(#[from] sqlx::Error),

    #[error("{0}")]
    ViolatedAssertion(String),

    #[error("Unauthorized")]
    Unauthorized,
}

impl warp::reject::Reject for ApiError {}

/// Turn rejections into appropriate status codes
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(custom_error) = err.find::<ApiError>() {
        match custom_error {
            ApiError::DBError(db_error) => {
                error!("Database error: {:?}", db_error);
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }
            ApiError::ViolatedAssertion(assertion) => {
                error!("Violated assertion: {:?}", assertion);
                return Ok(StatusCode::INTERNAL_SERVER_ERROR);
            }
            ApiError::Unauthorized => {
                error!("Unauthorized access");
                return Ok(StatusCode::UNAUTHORIZED);
            }
        }
    }

    Err(err)
}
