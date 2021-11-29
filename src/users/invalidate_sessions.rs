use crate::authentication::{delete_all_auth_tokens};
use crate::error::ApiError;
use crate::util::get_cookie_headers;
use crate::users::{get_password, user_exists_and_is_active, verify_password, UserCredentials};
use chrono::Duration;
use sqlx::PgPool;
use warp::http::StatusCode;
use warp::Reply;

/// Delete all auth_token of user and override existing http-only cookies.
pub async fn invalidate_sessions(
    user: UserCredentials,
    user_id: i32,
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

    delete_all_auth_tokens(user_id, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_cookie_headers("", Duration::zero()))
}
