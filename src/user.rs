use crate::error::ApiError;
use crate::util::{get_auth_token, get_current_time};
use bcrypt::{hash, verify};
use log::{error, info, warn};
use serde::Deserialize;
use sqlx::{query, PgPool};
use warp::http::{Response, StatusCode};
use warp::hyper::header::SET_COOKIE;
use warp::hyper::Body;
use warp::reject;
use warp::Reply;

/// Cost of bcrypt hashing algorithm
const BCRYPT_COST: u32 = 12;

/// Time in seconds for a session token to expire: 2 Months.
const TOKEN_EXPIRATION: u64 = 60 * 60 * 24 * 60;

/// Default starting balance for new users
/// 0.5 CHF
const DEFAULT_BALANCE: i64 = 5000;

/// This request form is expected for signup and login calls.
#[derive(Deserialize)]
pub struct UserCredentials {
    name: String,
    password: String,
}

/// Sign up new user. This stores the user data in the db.
pub async fn signup(
    user: UserCredentials,
    pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Creating user {}", user.name);

    if user_exists(&pool, &user.name)
        .await
        .map_err(|e| reject::custom(e))?
    {
        warn!("User {} already exists", user.name);
        return Ok(StatusCode::CONFLICT);
    }

    let hashed_password = hash(user.password, BCRYPT_COST);

    if hashed_password.is_err() {
        return Ok(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let hashed_password = hashed_password.unwrap();

    let now = get_current_time();

    create_user(&pool, &user.name, &hashed_password, now as i64)
        .await
        .map_err(|e| reject::custom(e))?;

    Ok(StatusCode::OK)
}

/// Log in existing user, this sets username and token cookies for future requests.
pub async fn login(
    user: UserCredentials,
    pool: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Login user {}", user.name);

    if !user_exists(&pool, &user.name)
        .await
        .map_err(|e| reject::custom(e))?
    {
        warn!("User {} doesn't exists", user.name);
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let password = get_password(&pool, &user.name)
        .await
        .map_err(|e| reject::custom(e))?;

    match verify_password(&user.name, &user.password, &password)
        .await
        .map_err(|e| reject::custom(e))?
    {
        false => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        true => (),
    }

    let now = get_current_time();

    let token = get_auth_token();

    store_token(&pool, &user.name, &token, now)
        .await
        .map_err(|e| reject::custom(e))?;

    Ok(get_cookie_headers(&user.name, &token))
}

async fn user_exists(pool: &PgPool, name: &str) -> Result<bool, ApiError> {
    let result = query!(
        "SELECT COUNT(id)
        FROM users 
        WHERE username = $1;",
        name
    )
    .fetch_one(pool)
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

async fn create_user(
    pool: &PgPool,
    name: &str,
    password_hash: &str,
    time: i64,
) -> Result<(), ApiError> {
    let query_result = query!(
        "INSERT INTO users (username, password, created_at, balance)
        VALUES ($1, $2, $3, $4);",
        name,
        password_hash,
        time,
        DEFAULT_BALANCE
    )
    .execute(pool)
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

/// Retrieve stored password hash for existing user.
pub async fn get_password(pool: &PgPool, name: &str) -> Result<String, ApiError> {
    let result = query!(
        "SELECT password
        FROM users 
        WHERE username = $1;",
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(result.password)
}

/// Verify supplied password for user.
async fn verify_password(name: &str, password: &str, hash: &str) -> Result<bool, ApiError> {
    match verify(password, &hash) {
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

/// Add a new token to the user. User is expected to exist.
pub async fn store_token(
    pool: &PgPool,
    name: &str,
    token: &str,
    created_at: u64,
) -> Result<(), ApiError> {
    let query_result = query!(
        "INSERT INTO auth_tokens (token, created_at, user_id)
        VALUES ($1, $2, (SELECT id FROM users WHERE username=$3));",
        token,
        created_at as i64,
        name
    )
    .execute(pool)
    .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 1 {
                Ok(())
            } else {
                Err(ApiError::ViolatedAssertion(
                    "Multiple rows affected in token storing".to_string(),
                ))
            }
        }
        Err(error) => Err(ApiError::DBError(error)),
    }
}

/// Get properly formatted cookie headers from name and token.
fn get_cookie_headers(name: &str, token: &str) -> Response<Body> {
    let response = Response::builder().status(StatusCode::OK).header(
        SET_COOKIE,
        format!("token={};HttpOnly;Max-Age={}", token, TOKEN_EXPIRATION),
    );

    response.body(Body::empty()).unwrap()
}
