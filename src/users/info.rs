use crate::{
    error::ApiError,
    users::{get_user_info, BALANCE_SCALE_FACTOR, DAILY_BALANCE_COST},
};
use log::info;
use serde::Serialize;
use sqlx::PgPool;

/// Response to User info request
#[derive(Serialize)]
pub struct UserInfoResponse {
    balance: f64,
    salt: Option<String>,
    remaining_days: f64,
}

/// Get user info
pub async fn user_info_handler(user_id: i32, db: PgPool) -> Result<impl warp::Reply, ApiError> {
    info!("Get info info of user {}", user_id);

    let user_info = get_user_info(user_id, &db).await?;
    let remaining_days = user_info.balance as f64 / DAILY_BALANCE_COST as f64;

    Ok(warp::reply::json(&UserInfoResponse {
        balance: user_info.balance as f64 / BALANCE_SCALE_FACTOR as f64,
        remaining_days,
        salt: user_info.salt,
    }))
}
