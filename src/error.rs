use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Database error")]
  DBError(#[from] sqlx::Error),

  #[error("{0}")]
  ViolatedAssertion(String),
}

impl warp::reject::Reject for ApiError {}
