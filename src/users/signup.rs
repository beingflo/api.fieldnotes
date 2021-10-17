use crate::error::ApiError;
use crate::users::{user_exists, BCRYPT_COST, DEFAULT_BALANCE};
use bcrypt::hash;
use chrono::{DateTime, Utc};
use log::{info, warn};
use serde::Deserialize;
use sqlx::{query, PgPool};
use warp::http::StatusCode;

/// This request form is expected for signupg calls.
#[derive(Deserialize)]
pub struct SignupCredentials {
    name: String,
    password: String,
    email: Option<String>,
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "event", rename_all = "lowercase")]
enum Event {
    StartTextli,
    PauseTextli,
    AddFunds,
}

/// Sign up new user. This stores the user data in the db.
pub async fn signup_handler(
    user: SignupCredentials,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Creating user {}", user.name);

    if user_exists(&user.name, &db).await? {
        warn!("User {} already exists", user.name);
        return Ok(StatusCode::CONFLICT);
    }

    let hashed_password = hash(user.password, BCRYPT_COST);

    if hashed_password.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let hashed_password = hashed_password.unwrap();

    let now = Utc::now();

    store_user(&user.name, &hashed_password, user.email, now, &db).await?;

    Ok(StatusCode::OK)
}

async fn store_user(
    name: &str,
    password_hash: &str,
    email: Option<String>,
    time: DateTime<Utc>,
    db: &PgPool,
) -> Result<(), ApiError> {
    let result = query!(
        "INSERT INTO users (username, password, email, created_at, balance)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id;",
        name,
        password_hash,
        email,
        time,
        DEFAULT_BALANCE
    )
    .fetch_one(db)
    .await?;

    let user_id = result.id;

    query!(
        "INSERT INTO transactions (user_id, event, date)
        VALUES ($1, $2, $3);",
        user_id,
        Event::StartTextli as Event,
        time,
    )
    .execute(db)
    .await?;

    Ok(())
}
