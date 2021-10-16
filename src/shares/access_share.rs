use crate::{error::ApiError, shares::get_share_expiration};
use chrono::{DateTime, Utc};
use log::{info, warn};
use serde::{Serialize};
use sqlx::{PgPool, query};

/// Request to create share
#[derive(Serialize)]
pub struct AccessShareResponse {
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    content: String,
    key: String,
}

pub async fn access_share_handler(
    token: String,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Accessing share");
    let expires_at= get_share_expiration(&token, &db).await?;

    let now = Utc::now();

    if let Some(expires) = expires_at {
      if expires < now {
          warn!("Share expired {}", token);

          return Err(warp::reject::custom(ApiError::Unauthorized))
      }
    }

    let note = access_share(&token, &db).await?;

    Ok(warp::reply::json(&note))
}

async fn access_share(token: &str, db: &PgPool) -> Result<AccessShareResponse, ApiError> {
    match query!(
        "SELECT notes.created_at, notes.modified_at, notes.content, notes.key 
        FROM shares 
        INNER JOIN notes ON shares.note_id = notes.id
        WHERE shares.token = $1;",
        token
    )
    .fetch_optional(db)
    .await?
    {
        Some(row) => Ok(AccessShareResponse {
            created_at: row.created_at,
            modified_at: row.modified_at,
            content: row.content,
            key: row.key,
        }),
        None => {
            warn!("Invalid share token {}", token);
            Err(ApiError::Unauthorized)
        }
    }
}
