use crate::authentication::{store_auth_token, TOKEN_EXPIRATION_WEEKS};
use crate::error::AppError;
use crate::users::{get_password, user_exists_and_is_active, verify_password, UserCredentials};
use crate::util::{get_auth_token, get_header_with_token};
use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Duration, Utc};
use hyper::StatusCode;
use sqlx::PgPool;

use super::get_user_id;

/// Log in existing user, this sets username and token cookies for future requests.
pub async fn login_handler(
    Json(user): Json<UserCredentials>,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    if !user_exists_and_is_active(&user.name, &db).await? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let id = get_user_id(&user.name, &db).await?;
    let password = get_password(id, &db).await?;

    match verify_password(&user.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        true => (),
    }

    let now = Utc::now();

    let token = get_auth_token();

    store_auth_token(&user.name, &token, now, &db).await?;

    let headers = get_header_with_token(&token, Duration::weeks(TOKEN_EXPIRATION_WEEKS));

    Ok(headers.into_response())
}
