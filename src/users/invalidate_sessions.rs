use crate::authentication::{delete_all_auth_tokens, AuthenticatedUser};
use crate::error::AppError;
use crate::users::UserCredentials;
use crate::util::get_header_with_token;
use axum::extract::Extension;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::Duration;
use hyper::StatusCode;
use sqlx::PgPool;

use super::validate_user_with_credentials;

/// Delete all auth_token of user and override existing http-only cookies.
pub async fn invalidate_sessions(
    Json(credentials): Json<UserCredentials>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    if !validate_user_with_credentials(
        &user.username,
        user.user_id,
        &credentials.name,
        &credentials.password,
        &db,
    )
    .await?
    {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    delete_all_auth_tokens(user.user_id, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_header_with_token("", Duration::zero()).into_response())
}
