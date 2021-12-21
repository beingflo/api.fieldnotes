use axum::{
    extract::{Extension, Query},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{query, PgPool};
use std::collections::HashMap;
use tokio_stream::StreamExt;

use crate::{authentication::AuthenticatedUser, error::AppError};

/// Response to list notes request
#[derive(Serialize)]
pub struct ListNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    metadata: String,
    key: String,
}

/// Response to list notes request
#[derive(Serialize)]
pub struct ListDeletedNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    deleted_at: DateTime<Utc>,
    metadata: String,
    key: String,
}

/// List all non-deleted notes
pub async fn list_notes_handler(
    Query(queries): Query<HashMap<String, String>>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    if queries.get("deleted").is_some() {
        let notes = list_deleted_notes(user.user_id, &db).await?;

        Ok(Json(notes).into_response())
    } else {
        let notes = list_notes(user.user_id, &db).await?;

        Ok(Json(notes).into_response())
    }
}

async fn list_notes(user_id: i32, db: &PgPool) -> Result<Vec<ListNoteResponse>, AppError> {
    let mut rows = query!(
        "SELECT token, created_at, modified_at, metadata, key
        FROM notes
        WHERE user_id = $1 AND deleted_at IS NULL",
        user_id
    )
    .fetch(db);

    let mut notes: Vec<ListNoteResponse> = Vec::new();

    while let Some(note) = rows.try_next().await? {
        notes.push(ListNoteResponse {
            id: note.token,
            modified_at: note.modified_at,
            created_at: note.created_at,
            metadata: note.metadata,
            key: note.key,
        });
    }

    Ok(notes)
}

async fn list_deleted_notes(
    user_id: i32,
    db: &PgPool,
) -> Result<Vec<ListDeletedNoteResponse>, AppError> {
    let mut rows = query!(
        "SELECT token, created_at, modified_at, deleted_at, metadata, key
        FROM notes
        WHERE user_id = $1 AND deleted_at IS NOT NULL",
        user_id
    )
    .fetch(db);

    let mut notes: Vec<ListDeletedNoteResponse> = Vec::new();

    while let Some(note) = rows.try_next().await? {
        notes.push(ListDeletedNoteResponse {
            id: note.token,
            modified_at: note.modified_at,
            created_at: note.created_at,
            deleted_at: note.deleted_at.unwrap(),
            metadata: note.metadata,
            key: note.key,
        });
    }

    Ok(notes)
}
