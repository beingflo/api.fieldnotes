use crate::{shares::get_share_expiration, shares::KeyJson, error::AppError};
use axum::{response::{Response, IntoResponse}, Json, extract::{Path, Extension}};
use chrono::{DateTime, Utc};
use log::error;
use serde::Serialize;
use sqlx::{query, PgPool};

/// Request to create share
#[derive(Serialize)]
pub struct AccessShareResponse {
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    content: String,
    iv: String,
}

pub async fn access_share_handler(Path(token): Path<String>, db: Extension<PgPool>) -> Result<Response, AppError> {
    let expires_at = get_share_expiration(&token, &db).await?;

    let now = Utc::now();

    if let Some(expires) = expires_at {
        if expires < now {
            return Err(AppError::Unauthorized);
        }
    }

    let note = access_share(&token, &db).await?;

    Ok(Json(&note).into_response())
}

async fn access_share(token: &str, db: &PgPool) -> Result<AccessShareResponse, AppError> {
    query!(
        "UPDATE shares
        SET view_count = view_count + 1
        WHERE shares.token = $1;",
        token
    )
    .execute(db)
    .await?;

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
        Some(row) => {
            let key: KeyJson = match serde_json::from_str(&row.key) {
                Ok(key) => key,
                Err(err) => {
                    error!("Serde error: {:?}", err);
                    return Err(AppError::ViolatedAssertion(
                        "Key field not serializable".to_string(),
                    ))
                }
            };

            Ok(AccessShareResponse {
                created_at: row.created_at,
                modified_at: row.modified_at,
                content: row.content,
                iv: key.iv_content,
            })
        }
        None => Err(AppError::Unauthorized),
    }
}
