mod access_share;
mod create_share;
mod delete_share;
mod list_publications;
mod list_shares;

pub use access_share::access_share_handler;
pub use create_share::create_share_handler;
pub use delete_share::delete_share_handler;
pub use list_publications::list_publications_handler;
pub use list_shares::list_shares_handler;

use crate::error::ApiError;
use chrono::{DateTime, Utc};
use log::warn;
use sqlx::{query, PgPool};

async fn get_share_expiration(token: &str, db: &PgPool) -> Result<Option<DateTime<Utc>>, ApiError> {
    match query!(
        "SELECT expires_at
        FROM shares 
        WHERE token= $1",
        token
    )
    .fetch_optional(db)
    .await?
    {
        Some(row) => Ok(row.expires_at),
        None => {
            warn!("Invalid share token {}", token);
            Err(ApiError::Unauthorized)
        }
    }
}
