use crate::{authentication::AuthenticatedUser, error::AppError, users::get_user_info};
use axum::{extract::Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

/// Response to User info request
#[derive(Serialize)]
pub struct UserInfoResponse {
    salt: Option<String>,
    username: String,
    email: Option<String>,
}

/// Get user info
pub async fn user_info_handler(
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Json<UserInfoResponse>, AppError> {
    let user_info = get_user_info(user.user_id, &db).await?;

    Ok(Json(UserInfoResponse {
        salt: user_info.salt,
        username: user_info.username,
        email: user_info.email,
    }))
}
