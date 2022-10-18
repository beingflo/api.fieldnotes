mod change_password;
mod delete_user;
mod info;
mod invalidate_sessions;
mod login;
mod logout;
mod salt;
mod signup;

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

/// Cost of bcrypt hashing algorithm
const BCRYPT_COST: u32 = 12;

/// This request form is expected for login calls.
#[derive(Deserialize)]
pub struct UserCredentials {
    name: String,
    password: String,
}

pub struct UserInfo {
    salt: Option<String>,
    username: String,
    email: Option<String>,
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

    Ok(UserInfo {
        salt: info.salt,
        username: info.username,
        email: info.email,
    })
}

async fn user_exists(name: &str, db: &PgPool) -> Result<bool, AppError> {
    let row = query!(
        "SELECT COUNT(id)
        FROM users 
        WHERE lower(username) = lower($1);",
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
    let forbidden = ";/?:@&=+$,#*[]{}()^|";

    name.chars().all(|c| !forbidden.contains(c))
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

    if !user_exists_and_is_active(credential_name, db).await? {
        return Ok(false);
    }

    let password = get_password(user_id, db).await?;

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
