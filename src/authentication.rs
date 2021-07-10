use crate::error::ApiError;
use crate::user::TOKEN_EXPIRATION;
use crate::util::get_current_time;
use log::{error, info, warn};
use sqlx::{query, PgPool};
use warp::reject;
use warp::Reply;

/// Checks if user has proper authorization for request.
pub async fn is_authorized(token: String, db: PgPool) -> Result<i32, warp::Rejection> {
    let result = query!(
        "SELECT user_id, created_at
        FROM auth_tokens 
        WHERE token = $1",
        token
    )
    .fetch_optional(&db)
    .await
    .map_err(|e| reject::custom(ApiError::DBError(e)))?;

    let (user_id, created_at) = match result {
        Some(tok) => (tok.user_id, tok.created_at),
        None => {
            warn!("Invalid token {}", token);
            return Err(warp::reject::custom(ApiError::Unauthorized));
        }
    };

    let now = get_current_time();

    if created_at + TOKEN_EXPIRATION > now {
        info!("Token valid for user {}", user_id);
        Ok(user_id)
    } else {
        warn!("Token expired for user {}", user_id);
        // TODO delete token and overwrite cookie
        Err(warp::reject::custom(ApiError::Unauthorized))
    }
}
