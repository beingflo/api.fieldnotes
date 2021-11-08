use crate::authentication::{delete_all_auth_tokens};
use crate::error::ApiError;
use crate::util::get_cookie_headers;
use chrono::Duration;
use sqlx::PgPool;

/// Delete all auth_token of user and override existing http-only cookies.
pub async fn invalidate_sessions(
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
    delete_all_auth_tokens(user_id, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_cookie_headers("", Duration::zero()))
}
