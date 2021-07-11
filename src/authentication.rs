use crate::error::ApiError;
use crate::user::TOKEN_EXPIRATION;
use crate::util::get_current_time;
use log::{info, warn};
use sqlx::{query, PgPool};
use warp::reject;

/// Checks if user has proper authorization for request.
pub async fn is_authorized(token: String, db: PgPool) -> Result<i32, warp::Rejection> {
    let (user_id, created_at) = get_auth_token_info(&token, &db).await?;

    let now = get_current_time();

    if created_at + TOKEN_EXPIRATION > now {
        info!("Token valid for user {}", user_id);
        Ok(user_id)
    } else {
        warn!("Token expired for user {}", user_id);

        delete_auth_token(&token, &db).await?;

        Err(warp::reject::custom(ApiError::Unauthorized))
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

// Delete provided auth token from db
async fn delete_auth_token(token: &str, db: &PgPool) -> Result<(), ApiError> {
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
