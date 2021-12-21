use crate::{util::get_note_token, error::AppError, authentication::AuthenticatedUser};
use axum::{Json, response::{IntoResponse, Response}, extract::Extension};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

/// Request to save note
#[derive(Deserialize)]
pub struct SaveNoteRequest {
    metadata: String,
    key: String,
    content: String,
}

/// Response to save note
#[derive(Serialize)]
pub struct SaveNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

/// Save a new note
pub async fn save_note_handler(
    user: AuthenticatedUser,
    Json(note): Json<SaveNoteRequest>,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    let now = Utc::now();
    let token = get_note_token();

    let SaveNoteRequest {
        metadata,
        key,
        content,
    } = note;

    save_note(user.user_id, &token, now, now, &metadata, &key, &content, &db).await?;

    Ok(Json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        created_at: now,
    }).into_response())
}

async fn save_note(
    user_id: i32,
    token: &str,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    metadata: &str,
    key: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), AppError> {
    query!(
        "INSERT INTO notes (token, user_id, created_at, modified_at, metadata, key, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7);",
        token,
        user_id,
        created_at,
        modified_at,
        metadata,
        key,
        content,
    )
    .execute(db)
    .await?;

    Ok(())
}
