use crate::error::ApiError;
use crate::util::get_note_token;
use chrono::{DateTime, Utc};
use log::info;
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
    user_id: i32,
    note: SaveNoteRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Saving note for user {}", user_id);

    let now = Utc::now();
    let token = get_note_token();

    let SaveNoteRequest {
        metadata,
        key,
        content,
    } = note;

    store_note(user_id, &token, now, now, &metadata, &key, &content, &db).await?;

    Ok(warp::reply::json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        created_at: now,
    }))
}

async fn store_note(
    user_id: i32,
    token: &str,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    metadata: &str,
    key: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
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
