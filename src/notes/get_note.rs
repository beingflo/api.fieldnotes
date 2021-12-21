use axum::{response::{Response, IntoResponse}, Json, extract::{Path, Extension}};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{query, PgPool};

use crate::{error::AppError, authentication::AuthenticatedUser};

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
    Path(token): Path<String>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    let note: GetNoteResponse = get_note(user.user_id, &token, &db).await?;

    Ok(Json(&note).into_response())
}

async fn get_note(user_id: i32, token: &str, db: &PgPool) -> Result<GetNoteResponse, AppError> {
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
        None => Err(AppError::Unauthorized),
    }
}
