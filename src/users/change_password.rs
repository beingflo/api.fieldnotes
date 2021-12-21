use crate::{users::BCRYPT_COST, error::AppError, authentication::{AuthenticatedUser}};
use axum::{Json, extract::Extension};
use bcrypt::hash;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::{query, PgPool};

use super::validate_user_with_credentials;

/// This request form is expected for changing password
#[derive(Deserialize)]
pub struct PasswordChangeRequest {
    name: String,
    password: String,
    password_new: String,
}

/// Change password of existing user
pub async fn change_password_handler(
    Json(credentials): Json<PasswordChangeRequest>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<StatusCode, AppError> {
    if !validate_user_with_credentials(&user.username, user.user_id, &credentials.name, &credentials.password, &db).await? {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let hashed_password = hash(credentials.password_new, BCRYPT_COST);

    if hashed_password.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let hashed_password = hashed_password.unwrap();

    change_password(user.user_id, &hashed_password, &db).await?;

    Ok(StatusCode::OK)
}

// Update password of existing user
async fn change_password(user_id: i32, password_hash: &str, db: &PgPool) -> Result<(), AppError> {
    let result = query!(
        "UPDATE users 
        SET password = $1
        WHERE id = $2",
        password_hash,
        user_id,
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AppError::ViolatedAssertion(
            "No rows affected when changing password".to_string(),
        ))
    }
}
