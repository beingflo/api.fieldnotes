use crate::{authentication::AuthenticatedFundedUser, error::AppError, util::get_note_token};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
    Json,
};
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
    user: AuthenticatedFundedUser,
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

    query!(
        "INSERT INTO notes (token, user_id, created_at, modified_at, metadata, key, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7);",
        token,
        user.user_id,
        now,
        now,
        metadata,
        key,
        content,
    )
    .execute(&*db)
    .await?;

    Ok(Json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        created_at: now,
    })
    .into_response())
}