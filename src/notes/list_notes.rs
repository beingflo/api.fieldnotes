use crate::error::ApiError;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Serialize};
use sqlx::{PgPool, query};
use std::collections::HashMap;
use tokio_stream::StreamExt;

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
    queries: HashMap<String, String>,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    if queries.get("deleted").is_some() {
        info!("Listing deleted notes for user {}", user_id);

        let notes = list_deleted_notes(user_id, &db).await?;

        Ok(warp::reply::json(&notes))
    } else {
        info!("Listing notes for user {}", user_id);

        let notes = list_notes(user_id, &db).await?;

        Ok(warp::reply::json(&notes))
    }
}

async fn list_notes(user_id: i32, db: &PgPool) -> Result<Vec<ListNoteResponse>, ApiError> {
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
) -> Result<Vec<ListDeletedNoteResponse>, ApiError> {
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