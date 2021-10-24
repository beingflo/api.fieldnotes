use crate::{error::ApiError, util::trucate_auth_token};
use chrono::{DateTime, Duration, Utc};
use log::info;
use sqlx::{query, PgPool};
use warp::reject;

/// Token expiration time: 2 months
pub const TOKEN_EXPIRATION_WEEKS: i64 = 8;

/// Checks if user has proper authorization token for request and return user id
/// used in further filters and handlers.
pub async fn is_authorized_with_user(token: String, db: PgPool) -> Result<i32, warp::Rejection> {
    let (user_id, created_at) = get_auth_token_info(&token, &db).await?;

    let now = Utc::now();

    info!("Access with token: {}", trucate_auth_token(&token));

    if created_at + Duration::weeks(TOKEN_EXPIRATION_WEEKS) > now {
        Ok(user_id)
    } else {
        Err(warp::reject::custom(ApiError::Unauthorized))
    }
}

// Get user_id and creation date of provided token
async fn get_auth_token_info(
    token: &str,
    db: &PgPool,
) -> Result<(i32, DateTime<Utc>), warp::Rejection> {
    match query!(
        "SELECT user_id, created_at
        FROM auth_tokens 
        WHERE token = $1",
        token
    )
    .fetch_optional(db)
    .await
    .map_err(|e| reject::custom(ApiError::DBError(e)))?
    {
        Some(tok) => Ok((tok.user_id, tok.created_at)),
        None => Err(warp::reject::custom(ApiError::Unauthorized)),
    }
}

/// Add a new token to the user. User is expected to exist.
pub async fn store_auth_token(
    name: &str,
    token: &str,
    created_at: DateTime<Utc>,
    db: &PgPool,
) -> Result<(), ApiError> {
    query!(
        "INSERT INTO auth_tokens (token, created_at, user_id)
        VALUES ($1, $2, (SELECT id FROM users WHERE username=$3));",
        token,
        created_at,
        name
    )
    .execute(db)
    .await?;

    Ok(())
}

// Delete provided auth token from db
pub async fn delete_auth_token(token: &str, db: &PgPool) -> Result<(), ApiError> {
    query!(
        "DELETE
        FROM auth_tokens 
        WHERE token = $1",
        &token
    )
    .execute(db)
    .await?;

    Ok(())
}
