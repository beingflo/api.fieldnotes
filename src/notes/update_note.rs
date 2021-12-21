use axum::{
    extract::{Extension, Path},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

use crate::{authentication::AuthenticatedUser, error::AppError};

/// Request to save note
#[derive(Deserialize)]
pub struct UpdateNoteRequest {
    metadata: String,
    key: String,
    content: String,
}

/// Response to update note
#[derive(Serialize)]
pub struct UpdateNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
}

/// Update an existing note
pub async fn update_note_handler(
    Path(token): Path<String>,
    user: AuthenticatedUser,
    Json(note): Json<UpdateNoteRequest>,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    let now = Utc::now();

    let UpdateNoteRequest {
        metadata,
        key,
        content,
    } = note;

    update_note(user.user_id, &token, now, &metadata, &key, &content, &db).await?;

    Ok(Json(&UpdateNoteResponse {
        id: token.clone(),
        modified_at: now,
    })
    .into_response())
}

async fn update_note(
    user_id: i32,
    token: &str,
    modified_at: DateTime<Utc>,
    metadata: &str,
    key: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), AppError> {
    let result = query!(
        "UPDATE notes
        SET modified_at = $1, metadata = $2, key = $3, content = $4
        WHERE user_id = $5 AND token = $6 AND deleted_at IS NULL",
        modified_at,
        metadata,
        key,
        content,
        user_id,
        token,
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}
