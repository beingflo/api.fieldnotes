use crate::authentication::{delete_all_auth_tokens, AuthenticatedUser};
use crate::error::AppError;
use crate::users::{get_password, user_exists_and_is_active, verify_password, UserCredentials};
use crate::util::get_header_with_token;
use axum::Json;
use axum::extract::Extension;
use axum::response::{Response, IntoResponse};
use chrono::Duration;
use hyper::StatusCode;
use sqlx::PgPool;

/// Delete all auth_token of user and override existing http-only cookies.
pub async fn invalidate_sessions(
    Json(credentials): Json<UserCredentials>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    if credentials.name != user.username {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    if !user_exists_and_is_active(&credentials.name, &db).await? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    let password = get_password(&credentials.name, &db).await?;

    match verify_password(&credentials.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        true => (),
    }

    delete_all_auth_tokens(user.user_id, &db).await?;

    // Set cookies empty and max-age 0 to force expiration
    Ok(get_header_with_token(&"", Duration::zero()).into_response())
}
