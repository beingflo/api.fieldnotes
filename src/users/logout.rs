use crate::{authentication::{delete_auth_token, AuthenticatedUser}, error::AppError, util::get_header_with_token};
use axum::{response::{Response, IntoResponse}, extract::Extension};
use chrono::Duration;
use sqlx::PgPool;

/// Log out user. This deletes auth_token and overrides existing http-only cookies.
pub async fn logout_handler(
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    delete_auth_token(&user.auth_token, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_header_with_token("", Duration::zero()).into_response())
}
