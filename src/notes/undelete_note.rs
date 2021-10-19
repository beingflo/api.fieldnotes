use crate::error::ApiError;
use chrono::{DateTime, Utc};
use log::info;
use serde::Serialize;
use sqlx::{query, PgPool};

/// Response to get note request
#[derive(Serialize)]
pub struct UndeleteNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    metadata: String,
    key: String,
    content: String,
}

/// Undelete an existing note
pub async fn undelete_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
    info!("Undeleting note for user {}", user_id);

    let note: UndeleteNoteResponse = undelete_note(user_id, &token, &db).await?;

    Ok(warp::reply::json(&note))
}

async fn undelete_note(
    user_id: i32,
    token: &str,
    db: &PgPool,
) -> Result<UndeleteNoteResponse, ApiError> {
    match query!(
        "UPDATE notes
        SET deleted_at = NULL
        WHERE user_id = $1 AND token = $2 AND deleted_at IS NOT NULL
        RETURNING token, created_at, modified_at, metadata, content, key",
        user_id,
        token,
    )
    .fetch_optional(db)
    .await?
    {
        Some(row) => Ok(UndeleteNoteResponse {
            id: token.to_string(),
            modified_at: row.modified_at,
            created_at: row.created_at,
            metadata: row.metadata,
            key: row.key,
            content: row.content,
        }),
        None => Err(ApiError::Unauthorized),
    }
}
