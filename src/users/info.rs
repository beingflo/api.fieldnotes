use crate::{
    users::{get_user_info, BALANCE_SCALE_FACTOR, DAILY_BALANCE_COST}, error::AppError, authentication::AuthenticatedUser,
};
use axum::{extract::Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

/// Response to User info request
#[derive(Serialize)]
pub struct UserInfoResponse {
    balance: f64,
    salt: Option<String>,
    remaining_days: f64,
    username: String,
    email: Option<String>,
}

/// Get user info
pub async fn user_info_handler(user: AuthenticatedUser, db: Extension<PgPool>) -> Result<Json<UserInfoResponse>, AppError> {
    let user_info = get_user_info(user.user_id, &db).await?;
    let remaining_days = user_info.balance as f64 / DAILY_BALANCE_COST as f64;

    Ok(Json(UserInfoResponse {
        balance: user_info.balance as f64 / BALANCE_SCALE_FACTOR as f64,
        remaining_days,
        salt: user_info.salt,
        username: user_info.username,
        email: user_info.email,
    }))
}
