use crate::authentication::delete_auth_token;
use crate::util::get_cookie_headers;
use chrono::Duration;
use log::info;
use sqlx::PgPool;

/// Log out user. This deletes auth_token and overrides existing http-only cookies.
pub async fn logout_handler(
    user_id: i32,
    token: String,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Log out user {}", user_id);

    delete_auth_token(&token, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_cookie_headers("", Duration::zero()))
}
