use crate::{
    error::AppError,
    users::{user_exists, BCRYPT_COST},
};
use axum::http::StatusCode;
use axum::{extract::Extension, Json};
use bcrypt::hash;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query, PgPool};

use super::username_valid;

/// This request form is expected for signupg calls.
#[derive(Deserialize)]
pub struct SignupCredentials {
    name: String,
    password: String,
    email: Option<String>,
}

/// Sign up new user. This stores the user data in the db.
pub async fn signup_handler(
    Json(user): Json<SignupCredentials>,
    db: Extension<PgPool>,
) -> Result<StatusCode, AppError> {
    if user_exists(&user.name, &db).await? {
        return Ok(StatusCode::CONFLICT);
    }

    if !username_valid(&user.name) {
        return Ok(StatusCode::UNPROCESSABLE_ENTITY);
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
) -> Result<(), AppError> {
    let mut tx = db.begin().await?;

    query!(
        "INSERT INTO users (username, password, email, created_at)
        VALUES ($1, $2, $3, $4);",
        name,
        password_hash,
        email,
        time,
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
