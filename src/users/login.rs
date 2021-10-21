use crate::authentication::{store_auth_token, TOKEN_EXPIRATION_WEEKS};
use crate::error::ApiError;
use crate::users::{get_password, user_exists_and_is_active, verify_password, UserCredentials};
use crate::util::{get_auth_token, get_cookie_headers};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use warp::http::StatusCode;
use warp::Reply;

/// Log in existing user, this sets username and token cookies for future requests.
pub async fn login_handler(
    user: UserCredentials,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
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

    Ok(get_cookie_headers(
        &token,
        Duration::weeks(TOKEN_EXPIRATION_WEEKS),
    ))
}
