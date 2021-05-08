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

  match user_exists(&pool, &user.name).await {
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

async fn user_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
  let result = query!(
    "SELECT COUNT(id)
    FROM users 
    WHERE username = $1;",
    name
  )
  .fetch_one(pool)
  .await;

  match result {
    Ok(row) => {
      if let Some(0) = row.count {
        Ok(false)
      } else {
        info!("User already exists");
        Ok(true)
      }
    }
    Err(error) => Err(error),
  }
}

async fn create_user(
  pool: &PgPool,
  name: &str,
  password_hash: &str,
  time: i64,
) -> Result<(), sqlx::Error> {
  let query_result = query!(
    "INSERT INTO users (username, password, created_at)
     VALUES ($1, $2, $3);",
    name,
    password_hash,
    time
  )
  .execute(pool)
  .await;

  // TODO Fix !== 1 case
  match query_result {
    Ok(result) => {
      if result.rows_affected() == 1 {
        Ok(())
      } else {
        Ok(())
      }
    }
    Err(error) => Err(error),
  }
}
