use crate::util::{get_current_time, get_secure_token};
use bcrypt::{hash, verify};
use log::{info, warn};
use serde::Deserialize;
use sqlx::{query, PgPool};
use warp::http::{Response, StatusCode};

/// Cost of bcrypt hashing algorithm. Low due to compute power on the target platform.
const BCRYPT_COST: u32 = 4;

/// Time in seconds for a session token to expire: 2 Months.
const TOKEN_EXPIRATION: u64 = 60 * 60 * 24 * 60;

/// This request form is expected for signup and login calls.
#[derive(Deserialize)]
pub struct UserCredentials {
  name: String,
  password: String,
}

/// Sign up new user. This stores the user data in the db.
pub async fn signup(
  user: UserCredentials,
  pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
  info!("Creating user {}", user.name);

  match user_exists(&user.name).await {
    Err(_) => return Ok(StatusCode::SERVICE_UNAVAILABLE),
    Ok(true) => {
      warn!("User {} already exists", user.name);

      return Ok(StatusCode::CONFLICT);
    }
    Ok(false) => {}
  }

  let hashed_password = hash(user.password, BCRYPT_COST);

  if hashed_password.is_err() {
    return Ok(StatusCode::INTERNAL_SERVER_ERROR);
  }

  let hashed_password = hashed_password.unwrap();
  println!("{}", hashed_password.len());

  let now = get_current_time();

  create_user(&pool, &user.name, &hashed_password, now).await;

  Ok(StatusCode::OK)
}

async fn user_exists(name: &str) -> Result<bool, ()> {
  // TODO
  Ok(false)
}

async fn create_user(pool: &PgPool, name: &str, password_hash: &str, time: i64) {
  query!(
    "INSERT INTO users (username, password, created_at)
     VALUES ($1, $2, $3);",
    name,
    password_hash,
    time
  )
  .execute(pool)
  .await
  .unwrap();
  // TODO error handling
}
