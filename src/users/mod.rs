mod change_password;
mod delete_user;
mod info;
mod invalidate_sessions;
mod login;
mod logout;
mod salt;
mod signup;
use chrono::Utc;

pub use change_password::change_password_handler;
pub use delete_user::delete_user_handler;
pub use info::user_info_handler;
pub use invalidate_sessions::invalidate_sessions;
pub use login::login_handler;
pub use logout::logout_handler;
pub use salt::store_salt_handler;
pub use signup::signup_handler;

use crate::error::AppError;
use bcrypt::verify;
use log::error;
use serde::Deserialize;
use sqlx::{query, PgPool};

/// Balance is stored as CHF * 10^6 to avoid significant rounding errors
pub const BALANCE_SCALE_FACTOR: i64 = 1_000_000;

/// Default starting balance for new users
/// Increased for beta TODO revert later
pub const DEFAULT_BALANCE: i64 = 20_000_000;

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
    email: Option<String>,
}

#[derive(sqlx::Type, Debug)]
#[sqlx(type_name = "event", rename_all = "lowercase")]
enum TransactionEvent {
    StartFieldnotes,
    PauseFieldnotes,
    AddFunds,
}

pub async fn is_funded(user_id: i32, db: &PgPool) -> Result<(), AppError> {
    let user_info = get_user_info(user_id, &db).await?;

    if user_info.balance > FUNDED_BALANCE {
        Ok(())
    } else {
        Err(AppError::Underfunded)
    }
}

async fn get_user_info(user_id: i32, db: &PgPool) -> Result<UserInfo, AppError> {
    let info = query!(
        "SELECT username, salt, email
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
                return Err(AppError::ViolatedAssertion(
                    "Transactions corrupted: Subsquent start dates".into(),
                ));
            }
            start_date = Some(transaction.date);
        }

        if matches!(transaction.event, TransactionEvent::PauseFieldnotes) {
            if start_date.is_none() {
                return Err(AppError::ViolatedAssertion(
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
        email: info.email,
    })
}

async fn user_exists(name: &str, db: &PgPool) -> Result<bool, AppError> {
    let row = query!(
        "SELECT COUNT(id)
        FROM users 
        WHERE username = $1;",
        name
    )
    .fetch_one(db)
    .await?;

    if let Some(0) = row.count {
        Ok(false)
    } else {
        Ok(true)
    }
}

fn username_valid(name: &str) -> bool {
    let forbidden = ";/?:@&=+$,#*[]{}|";

    if name.chars().all(|c| !forbidden.contains(c)) {
        true
    } else {
        false
    }
}

pub async fn user_exists_and_is_active(name: &str, db: &PgPool) -> Result<bool, AppError> {
    let row = query!(
        "SELECT COUNT(id)
        FROM users 
        WHERE username = $1 AND deleted_at IS NULL;",
        name
    )
    .fetch_one(db)
    .await?;

    if let Some(0) = row.count {
        Ok(false)
    } else {
        Ok(true)
    }
}

pub async fn validate_user_with_credentials(
    username: &str,
    user_id: i32,
    credential_name: &str,
    credential_password: &str,
    db: &PgPool,
) -> Result<bool, AppError> {
    if credential_name != username {
        return Ok(false);
    }

    if !user_exists_and_is_active(credential_name, &db).await? {
        return Ok(false);
    }

    let password = get_password(user_id, &db).await?;

    if !verify_password(credential_password, &password).await? {
        return Ok(false);
    }

    Ok(true)
}

/// Retrieve stored password hash for existing user.
pub async fn get_password(id: i32, db: &PgPool) -> Result<String, AppError> {
    let result = query!(
        "SELECT password
        FROM users 
        WHERE id = $1;",
        id
    )
    .fetch_one(db)
    .await?;

    Ok(result.password)
}

/// Retrieve stored password hash for existing user.
pub async fn get_user_id(name: &str, db: &PgPool) -> Result<i32, AppError> {
    let result = query!(
        "SELECT id
        FROM users 
        WHERE username = $1;",
        name
    )
    .fetch_one(db)
    .await?;

    Ok(result.id)
}

/// Verify supplied password for user.
async fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    match verify(password, hash) {
        Err(err) => {
            error!("Error while verifying password: {:?}", err);
            Err(AppError::ViolatedAssertion(
                "brcypt verify error".to_string(),
            ))
        }
        Ok(false) => Ok(false),
        Ok(true) => Ok(true),
    }
}
