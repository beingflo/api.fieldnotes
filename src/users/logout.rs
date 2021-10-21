use crate::authentication::delete_auth_token;
use crate::error::ApiError;
use crate::util::get_cookie_headers;
use chrono::Duration;
use sqlx::PgPool;

/// Log out user. This deletes auth_token and overrides existing http-only cookies.
pub async fn logout_handler(
    _userid: i32,
    token: String,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
    delete_auth_token(&token, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_cookie_headers("", Duration::zero()))
}
