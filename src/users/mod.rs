mod change_password;
mod delete_user;
mod info;
mod invalidate_sessions;
mod login;
mod logout;
mod salt;
mod signup;

pub use change_password::change_password_handler;
use chrono::Utc;
pub use delete_user::delete_user_handler;
pub use info::user_info_handler;
pub use invalidate_sessions::invalidate_sessions;
pub use login::login_handler;
pub use logout::logout_handler;
pub use salt::store_salt_handler;
pub use signup::signup_handler;

use crate::error::ApiError;
use bcrypt::verify;
use log::error;
use serde::Deserialize;
use sqlx::{query, PgPool};

/// Balance is stored as CHF * 10^6 to avoid significant rounding errors
pub const BALANCE_SCALE_FACTOR: i64 = 1_000_000;

/// Default starting balance for new users
pub const DEFAULT_BALANCE: i64 = 1_000_000;

/// Once balance falls below this threshold,
/// account becomes read only
pub const FUNDED_BALANCE: i64 = -500_000;

/// Cost per day calculated as: CHF 1 / Month = CHF 12 / Year
/// CHF 12 / 365 days per year = CHF 0.032876 / Day = CHF 0.0013698 / Hour
pub const HOURLY_BALANCE_COST: i64 = 1_369;
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
    balance: i64,
    salt: Option<String>,
    username: String,
}

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "event", rename_all = "lowercase")]
enum TransactionEvent {
    StartFieldnotes,
    PauseFieldnotes,
    AddFunds,
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
    let info = query!(
        "SELECT username, salt
        FROM users 
        WHERE id = $1;",
        user_id,
    )
    .fetch_one(db)
    .await?;

    let transactions = query!(
        r#"SELECT event AS "event!: TransactionEvent", amount, date
        FROM transactions
        WHERE user_id = $1
        ORDER BY date;"#,
        user_id,
    )
    .fetch_all(db)
    .await?;

    let mut credit = 0;
    let mut debit = 0;

    let mut start_date = None;

    for transaction in transactions {
        if matches!(transaction.event, TransactionEvent::AddFunds) {
            credit += transaction.amount.unwrap();
        }

        if matches!(transaction.event, TransactionEvent::StartFieldnotes) {
            if start_date.is_some() {
                return Err(ApiError::ViolatedAssertion(
                    "Transactions corrupted: Subsquent start dates".into(),
                ));
            }
            start_date = Some(transaction.date);
        }

        if matches!(transaction.event, TransactionEvent::PauseFieldnotes) {
            if start_date.is_none() {
                return Err(ApiError::ViolatedAssertion(
                    "Transactions corrupted: Pause event preceding start event".into(),
                ));
            }
            let duration = transaction.date - start_date.unwrap();
            let hours = duration.num_hours();

            let cost = hours * HOURLY_BALANCE_COST;
            debit += cost;

            start_date = None;
        }
    }

    if start_date.is_some() {
        let duration = Utc::now() - start_date.unwrap();
        let hours = duration.num_hours();

        let cost = hours * HOURLY_BALANCE_COST;
        debit += cost;
    }

    let balance = DEFAULT_BALANCE + credit - debit;

    Ok(UserInfo {
        balance,
        salt: info.salt,
        username: info.username,
    })
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
async fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    match verify(password, hash) {
        Err(err) => {
            error!("Error while verifying password: {:?}", err);
            Err(ApiError::ViolatedAssertion(
                "brcypt verify error".to_string(),
            ))
        }
        Ok(false) => Ok(false),
        Ok(true) => Ok(true),
    }
}
