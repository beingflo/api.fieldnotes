use crate::error::ApiError;
use crate::util::get_current_time;
use log::{info, warn};
use sqlx::{query, PgPool};
use warp::reject;

/// Time in seconds for a session token to expire: 2 Months.
pub const TOKEN_EXPIRATION: i64 = 60 * 60 * 24 * 60;

/// Checks if user has proper authorization for request.
pub async fn is_authorized(token: String, db: PgPool) -> Result<(), warp::Rejection> {
    let (user_id, created_at) = get_auth_token_info(&token, &db).await?;

    let now = get_current_time();

    if created_at + TOKEN_EXPIRATION > now {
        info!("Token valid for user {}", user_id);
        Ok(())
    } else {
        warn!("Token expired for user {}", user_id);

        delete_auth_token(&token, &db).await?;

        Err(warp::reject::custom(ApiError::Unauthorized))
    }
}

pub async fn get_user_id(token: String, db: PgPool) -> Result<i32, warp::Rejection> {
    match query!(
        "SELECT user_id
        FROM auth_tokens 
        WHERE token = $1",
        &token
    )
    .fetch_optional(&db)
    .await
    .map_err(|e| reject::custom(ApiError::DBError(e)))?
    {
        Some(tok) => Ok(tok.user_id),
        None => {
            warn!("Invalid token {}", token);
            return Err(warp::reject::custom(ApiError::Unauthorized));
        }
    }
}

// Get user_id and creation date of provided token
async fn get_auth_token_info(token: &str, db: &PgPool) -> Result<(i32, i64), warp::Rejection> {
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
        None => {
            warn!("Invalid token {}", token);
            return Err(warp::reject::custom(ApiError::Unauthorized));
        }
    }
}

/// Add a new token to the user. User is expected to exist.
pub async fn store_auth_token(
    db: &PgPool,
    name: &str,
    token: &str,
    created_at: i64,
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
