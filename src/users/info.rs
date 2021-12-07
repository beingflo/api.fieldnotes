use crate::{
    error::ApiError,
    users::{get_user_info, BALANCE_SCALE_FACTOR, DAILY_BALANCE_COST},
};
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
pub async fn user_info_handler(user_id: i32, db: PgPool) -> Result<impl warp::Reply, ApiError> {
    let user_info = get_user_info(user_id, &db).await?;
    let remaining_days = user_info.balance as f64 / DAILY_BALANCE_COST as f64;

    Ok(warp::reply::json(&UserInfoResponse {
        balance: user_info.balance as f64 / BALANCE_SCALE_FACTOR as f64,
        remaining_days,
        salt: user_info.salt,
        username: user_info.username,
        email: user_info.email,
    }))
}
