use axum::{Json, extract::Extension};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::{query, PgPool};

use crate::{error::AppError, authentication::AuthenticatedUser};

/// This request form is expected for storing salt
#[derive(Deserialize)]
pub struct UserSaltRequest {
    salt: String,
}

/// Store user salt
pub async fn store_salt_handler(
    user: AuthenticatedUser,
    Json(salt): Json<UserSaltRequest>,
    db: Extension<PgPool>,
) -> Result<StatusCode, AppError> {
    store_salt(user.user_id, &salt.salt, &db).await?;

    Ok(StatusCode::OK)
}

// Update password of existing user
async fn store_salt(user_id: i32, salt: &str, db: &PgPool) -> Result<(), AppError> {
    let result = query!(
        "UPDATE users 
        SET salt = $1
        WHERE id = $2",
        salt,
        user_id,
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AppError::ViolatedAssertion(
            "No rows affected when storing salt".to_string(),
        ))
    }
}
