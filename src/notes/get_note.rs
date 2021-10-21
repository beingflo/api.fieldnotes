use crate::error::ApiError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{query, PgPool};

/// Response to get note request
#[derive(Serialize)]
pub struct GetNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    metadata: String,
    key: String,
    content: String,
}

/// Get an existing note
pub async fn get_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
    let note: GetNoteResponse = get_note(user_id, &token, &db).await?;

    Ok(warp::reply::json(&note))
}

async fn get_note(user_id: i32, token: &str, db: &PgPool) -> Result<GetNoteResponse, ApiError> {
    match query!(
        "SELECT token, created_at, modified_at, metadata, key, content
        FROM notes
        WHERE user_id = $1 AND token = $2 AND deleted_at IS NULL",
        user_id,
        token,
    )
    .fetch_optional(db)
    .await?
    {
        Some(row) => Ok(GetNoteResponse {
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
