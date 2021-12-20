use crate::authentication::{store_auth_token, TOKEN_EXPIRATION_WEEKS};
use crate::error::AppError;
use crate::users::{get_password, user_exists_and_is_active, verify_password, UserCredentials};
use crate::util::{get_auth_token};
use axum::http::HeaderValue;
use axum::response::{Response, IntoResponse};
use axum::{Json, http};
use axum::extract::Extension;
use chrono::{Duration, Utc};
use hyper::{StatusCode, HeaderMap};
use sqlx::PgPool;

/// Log in existing user, this sets username and token cookies for future requests.
pub async fn login_handler(
    Json(user): Json<UserCredentials>,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    if !user_exists_and_is_active(&user.name, &db).await? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let password = get_password(&user.name, &db).await?;

    match verify_password(&user.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        true => (),
    }

    let now = Utc::now();

    let token = get_auth_token();

    store_auth_token(&user.name, &token, now, &db).await?;

    let cookie = format!(
        "token={};HttpOnly;Max-Age={}",
        token,
        Duration::weeks(TOKEN_EXPIRATION_WEEKS).num_seconds(),
    );

    let cookie_header = HeaderValue::from_str(&cookie).expect("Cookie value invalid");

    let mut headers = HeaderMap::new();
    headers.insert(http::header::SET_COOKIE, cookie_header);

    Ok(headers.into_response())
}
