use crate::authentication::{delete_auth_token, store_auth_token, TOKEN_EXPIRATION_WEEKS};
use crate::error::ApiError;
use crate::util::{get_auth_token, get_cookie_headers};
use bcrypt::{hash, verify};
use chrono::{DateTime, Duration, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};
use warp::http::StatusCode;
use warp::Reply;

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

/// This request form is expected for signupg calls.
#[derive(Deserialize)]
pub struct SignupCredentials {
    name: String,
    password: String,
    email: Option<String>,
}

pub struct UserInfo {
    balance: i64 ,
    salt: Option<String>,
}

/// This request form is expected for storing salt
#[derive(Deserialize)]
pub struct UserSaltRequest {
    salt: String,
}

/// This request form is expected for changing password
#[derive(Deserialize)]
pub struct PasswordChangeRequest {
    name: String,
    password: String,
    password_new: String,
}

/// Response to User info request
#[derive(Serialize)]
pub struct UserInfoResponse {
    balance: f64,
    salt: Option<String>,
    remaining_days: f64,
}

/// Sign up new user. This stores the user data in the db.
pub async fn signup(
    user: SignupCredentials,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Creating user {}", user.name);

    if user_exists(&user.name, &db).await? {
        warn!("User {} already exists", user.name);
        return Ok(StatusCode::CONFLICT);
    }

    let hashed_password = hash(user.password, BCRYPT_COST);

    if hashed_password.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let hashed_password = hashed_password.unwrap();

    let now = Utc::now();

    store_user(&user.name, &hashed_password, user.email, now, &db).await?;

    Ok(StatusCode::OK)
}

/// Delete user with all associated data
pub async fn delete_user(
    credentials: UserCredentials,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Deleting user {}", credentials.name);

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

    delete_all_user_data(user_id, &db).await?;

    Ok(StatusCode::OK)
}

/// Log in existing user, this sets username and token cookies for future requests.
pub async fn login(user: UserCredentials, db: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Login user {}", user.name);

    if !user_exists_and_is_active(&user.name, &db).await? {
        warn!("User {} doesn't exists", user.name);
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let password = get_password(&user.name, &db).await?;

    match verify_password(&user.name, &user.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        true => (),
    }

    let now = Utc::now();

    let token = get_auth_token();

    store_auth_token(&user.name, &token, now, &db).await?;

    Ok(get_cookie_headers(
        &token,
        Duration::weeks(TOKEN_EXPIRATION_WEEKS),
    ))
}

/// Change password of existing user
pub async fn change_password(
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

    update_password(user_id, &hashed_password, &db).await?;

    Ok(StatusCode::OK)
}

/// Get user info
pub async fn user_info_handler(
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Get info info of user {}", user_id);

    let user_info = get_user_info(user_id, &db).await?;
    let remaining_days = user_info.balance as f64 / DAILY_BALANCE_COST as f64;

    Ok(warp::reply::json(&UserInfoResponse {
        balance: user_info.balance as f64 / 1_000_000.0,
        remaining_days,
        salt: user_info.salt,
    }))
}

/// Store user salt
pub async fn store_salt_handler(
    user_id: i32,
    salt: UserSaltRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Store salt for user {}", user_id);

    store_salt(user_id, &salt.salt, &db).await?;

    Ok(StatusCode::OK)
}

/// Log out user. This deletes auth_token and overrides existing http-only cookies.
pub async fn logout(
    user_id: i32,
    token: String,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Log out user {}", user_id);

    delete_auth_token(&token, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_cookie_headers("", Duration::zero()))
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

// Update password of existing user
async fn store_salt(user_id: i32, salt: &str, db: &PgPool) -> Result<(), ApiError> {
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
        Err(ApiError::ViolatedAssertion(
            "Multiple rows affected when storing salt".to_string(),
        ))
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

async fn store_user(
    name: &str,
    password_hash: &str,
    email: Option<String>,
    time: DateTime<Utc>,
    db: &PgPool,
) -> Result<(), ApiError> {
    let query_result = query!(
        "INSERT INTO users (username, password, email, created_at, balance)
        VALUES ($1, $2, $3, $4, $5);",
        name,
        password_hash,
        email,
        time,
        DEFAULT_BALANCE
    )
    .execute(db)
    .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 1 {
                Ok(())
            } else {
                Err(ApiError::ViolatedAssertion(
                    "Multiple rows affected in user creation".to_string(),
                ))
            }
        }
        Err(error) => Err(ApiError::DBError(error)),
    }
}

// Update password of existing user
async fn update_password(user_id: i32, password_hash: &str, db: &PgPool) -> Result<(), ApiError> {
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

/// Delete all user data
pub async fn delete_all_user_data(user_id: i32, db: &PgPool) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;

    query!(
        "DELETE
        FROM notes
        WHERE user_id = $1;",
        user_id
    )
    .execute(&mut tx)
    .await?;

    query!(
        "DELETE
        FROM auth_tokens 
        WHERE user_id = $1;",
        user_id
    )
    .execute(&mut tx)
    .await?;

    let now = Utc::now();

    query!(
        "UPDATE users
        SET deleted_at = $1
        WHERE id = $2;",
        now,
        user_id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
