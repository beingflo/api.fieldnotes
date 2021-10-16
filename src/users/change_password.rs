use crate::error::ApiError;
use crate::users::{get_password, user_exists_and_matches_id, verify_password, BCRYPT_COST};
use bcrypt::hash;
use log::{info, warn};
use serde::Deserialize;
use sqlx::{query, PgPool};
use warp::http::StatusCode;

/// This request form is expected for changing password
#[derive(Deserialize)]
pub struct PasswordChangeRequest {
    name: String,
    password: String,
    password_new: String,
}

/// Change password of existing user
pub async fn change_password_handler(
    credentials: PasswordChangeRequest,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Change password for user {}", user_id);

    if !user_exists_and_matches_id(&credentials.name, user_id, &db).await? {
        warn!(
            "User {} doesn't exists or doesn't match auth token",
            user_id
        );
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let password = get_password(&credentials.name, &db).await?;

    match verify_password(&credentials.name, &credentials.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED),
        true => (),
    }

    let hashed_password = hash(credentials.password_new, BCRYPT_COST);

    if hashed_password.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let hashed_password = hashed_password.unwrap();

    change_password(user_id, &hashed_password, &db).await?;

    Ok(StatusCode::OK)
}

// Update password of existing user
async fn change_password(user_id: i32, password_hash: &str, db: &PgPool) -> Result<(), ApiError> {
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
        Err(ApiError::ViolatedAssertion(
            "Multiple rows affected when updating note".to_string(),
        ))
    }
}
