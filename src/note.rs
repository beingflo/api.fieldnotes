use crate::error::ApiError;
use crate::util::get_note_token;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query};
use std::collections::HashMap;
use tokio_stream::StreamExt;
use warp::{http::StatusCode};

/// Request to save note
#[derive(Deserialize)]
pub struct SaveRequest {
    metadata: String,
    key: String,
    public: bool,
    content: String,
}

/// Response to save note
#[derive(Serialize)]
pub struct SaveNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

/// Response to update note 
#[derive(Serialize)]
pub struct UpdateNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
}

/// Response to list notes request
#[derive(Serialize)]
pub struct ListNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    metadata: String,
    key: String,
    public: bool,
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
    public: bool,
}

/// Response to get note request
#[derive(Serialize)]
pub struct GetNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    metadata: String,
    key: String,
    public: bool,
    content: String,
}

/// Save a new note
pub async fn save_note_handler(
    user_id: i32,
    note: SaveRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Saving note for user {}", user_id);

    let now = Utc::now();
    let token = get_note_token();

    let SaveRequest {
        metadata,
        key,
        public,
        content,
    } = note;

    store_note(
        user_id,
        &token,
        now,
        now,
        &metadata,
        &key,
        public,
        &content,
        &db,
    )
    .await?;

    Ok(warp::reply::json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        created_at: now,
    }))
}

/// Get an existing note
pub async fn get_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Get note for user {}", user_id);

    let note: GetNoteResponse = get_note(user_id, &token, &db).await?;

    Ok(warp::reply::json(&note))
}

/// Delete an existing note
pub async fn delete_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Deleting note for user {}", user_id);

    let now = Utc::now();

    delete_note(user_id, &token, now, &db).await?;

    Ok(StatusCode::OK)
}

/// Undelete an existing note
pub async fn undelete_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Undeleting note for user {}", user_id);

    undelete_note(user_id, &token, &db).await?;

    Ok(StatusCode::OK)
}

/// Update an existing note
pub async fn update_note_handler(
    token: String,
    user_id: i32,
    note: SaveRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Updating note for user {}", user_id);

    let now = Utc::now();

    let SaveRequest {
        metadata,
        key,
        public,
        content,
    } = note;

    update_note(
        user_id,
        &token,
        now,
        &metadata,
        &key,
        public,
        &content,
        &db,
    )
    .await?;

    Ok(warp::reply::json(&UpdateNoteResponse {
        id: token.clone(),
        modified_at: now,
    }))
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

async fn get_note(user_id: i32, token: &str, db: &PgPool) -> Result<GetNoteResponse, ApiError> {
    match query!(
        "SELECT token, created_at, modified_at, metadata, key, public, content
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
            public: row.public,
            content: row.content,
        }),
        None => Err(ApiError::Unauthorized),
    }
}

async fn store_note(
    user_id: i32,
    token: &str,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    metadata: &str,
    key: &str,
    public: bool,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
    query!(
        "INSERT INTO notes (token, user_id, created_at, modified_at, metadata, key, public, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8);",
        token,
        user_id,
        created_at,
        modified_at,
        metadata,
        key,
        public,
        content,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn delete_note(
    user_id: i32,
    token: &str,
    deleted_at: DateTime<Utc>,
    db: &PgPool,
) -> Result<(), ApiError> {
    let result = query!(
        "UPDATE notes
        SET deleted_at = $1
        WHERE user_id = $2 AND token = $3 AND deleted_at IS NULL",
        deleted_at,
        user_id,
        token,
    )
    .execute(db)
    .await?;

    // Delete shares of this note
    query!(
        "DELETE 
        FROM shares 
        WHERE note_id = (
            SELECT id
            FROM notes
            WHERE token = $1 AND user_id = $2
        ) AND user_id = $2;",
        token,
        user_id
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else if result.rows_affected() == 0 {
        Err(ApiError::Unauthorized)
    } else {
        Err(ApiError::ViolatedAssertion(
            "Multiple rows affected when updating note".to_string(),
        ))
    }
}

async fn undelete_note(user_id: i32, token: &str, db: &PgPool) -> Result<(), ApiError> {
    let result = query!(
        "UPDATE notes
        SET deleted_at = NULL
        WHERE user_id = $1 AND token = $2 AND deleted_at IS NOT NULL",
        user_id,
        token,
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else if result.rows_affected() == 0 {
        Err(ApiError::Unauthorized)
    } else {
        Err(ApiError::ViolatedAssertion(
            "Multiple rows affected when updating note".to_string(),
        ))
    }
}

async fn update_note(
    user_id: i32,
    token: &str,
    modified_at: DateTime<Utc>,
    metadata: &str,
    key: &str,
    public: bool,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
    let result = query!(
        "UPDATE notes
        SET modified_at = $1, metadata = $2, key = $3, public = $4, content = $5
        WHERE user_id = $6 AND token = $7 AND deleted_at IS NULL",
        modified_at,
        metadata,
        key,
        public,
        content,
        user_id,
        token,
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else if result.rows_affected() == 0 {
        Err(ApiError::Unauthorized)
    } else {
        Err(ApiError::ViolatedAssertion(
            "Multiple rows affected when updating note".to_string(),
        ))
    }
}

async fn list_notes(user_id: i32, db: &PgPool) -> Result<Vec<ListNoteResponse>, ApiError> {
    let mut rows = query!(
        "SELECT token, created_at, modified_at, metadata, key, public 
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
            public: note.public,
        });
    }

    Ok(notes)
}

async fn list_deleted_notes(
    user_id: i32,
    db: &PgPool,
) -> Result<Vec<ListDeletedNoteResponse>, ApiError> {
    let mut rows = query!(
        "SELECT token, created_at, modified_at, deleted_at, metadata, key, public 
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
            public: note.public,
        });
    }

    Ok(notes)
}
