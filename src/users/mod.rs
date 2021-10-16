mod signup;
mod login;
mod logout;
mod info;
mod change_password;
mod delete_user;
mod salt;

use crate::error::ApiError;
use bcrypt::{verify};
use log::{error, info, warn};
use serde::{Deserialize};
use sqlx::{query, PgPool};

pub use signup::signup;
pub use login::login;
pub use logout::logout;
pub use delete_user::delete_user;
pub use info::user_info_handler;
pub use change_password::change_password;
pub use salt::store_salt_handler;

/// Balance is stored as CHF * 10^6 to avoid significant rounding errors

/// Default starting balance for new users
/// 1.0 CHF = 1'000'000
pub const DEFAULT_BALANCE: i64 = 1_000_000;

/// Once balance falls below this threshold,
/// account becomes read only
pub const FUNDED_BALANCE: i64 = -500_000;

/// Cost per day calculated as: CHF 1 / Month = CHF 12 / Year
/// CHF 12 / 365 days per year = CHF 0.032876 / Day
pub const DAILY_BALANCE_COST: i64 = 32_876;

/// Cost of bcrypt hashing algorithm
const BCRYPT_COST: u32 = 12;

/// This request form is expected for login calls.
#[derive(Deserialize)]
pub struct UserCredentials {
    name: String,
    password: String,
}

pub struct UserInfo {
    balance: i64 ,
    salt: Option<String>,
}


pub async fn is_funded(user_id: i32, db: PgPool) -> Result<(), warp::Rejection> {
    let user_info = get_user_info(user_id, &db).await?;

    if user_info.balance > FUNDED_BALANCE {
        Ok(())
    } else {
        Err(warp::reject::custom(ApiError::Underfunded))
    }
}

async fn get_user_info(user_id: i32, db: &PgPool) -> Result<UserInfo, ApiError> {
    match query!(
        "SELECT balance, salt
        FROM users 
        WHERE id = $1;",
        user_id,
    )
    .fetch_one(db)
    .await
    {
        Ok(row) => Ok(UserInfo { balance: row.balance, salt: row.salt }),
        Err(error) => Err(ApiError::DBError(error)),
    }
}

async fn user_exists(name: &str, db: &PgPool) -> Result<bool, ApiError> {
    let result = query!(
        "SELECT COUNT(id)
        FROM users 
        WHERE username = $1;",
        name
    )
    .fetch_one(db)
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
        Err(error) => Err(ApiError::DBError(error)),
    }
}

async fn user_exists_and_is_active(name: &str, db: &PgPool) -> Result<bool, ApiError> {
    let result = query!(
        "SELECT COUNT(id)
        FROM users 
        WHERE username = $1 AND deleted_at IS NULL;",
        name
    )
    .fetch_one(db)
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
        Err(error) => Err(ApiError::DBError(error)),
    }
}

async fn user_exists_and_matches_id(
    name: &str,
    user_id: i32,
    db: &PgPool,
) -> Result<bool, ApiError> {
    match query!(
        "SELECT id
        FROM users 
        WHERE username = $1 AND deleted_at IS NULL;",
        name
    )
    .fetch_optional(db)
    .await?
    {
        Some(row) => {
            if row.id == user_id {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        None => Err(ApiError::Unauthorized),
    }
}

/// Retrieve stored password hash for existing user.
pub async fn get_password(name: &str, db: &PgPool) -> Result<String, ApiError> {
    let result = query!(
        "SELECT password
        FROM users 
        WHERE username = $1;",
        name
    )
    .fetch_one(db)
    .await?;

    Ok(result.password)
}

/// Verify supplied password for user.
async fn verify_password(name: &str, password: &str, hash: &str) -> Result<bool, ApiError> {
    match verify(password, hash) {
        Err(err) => {
            error!("Error while verifying password: {:?}", err);
            Err(ApiError::ViolatedAssertion(
                "brcypt verify error".to_string(),
            ))
        }
        Ok(false) => {
            warn!("User {} supplied wrong password", name);
            Ok(false)
        }
        Ok(true) => {
            info!("User {} supplied correct password", name);
            Ok(true)
        }
    }
}
