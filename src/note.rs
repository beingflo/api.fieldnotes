use crate::endpoint::{get_note_endpoints, NoteEndpoints};
use crate::error::ApiError;
use crate::util::{get_current_time, get_note_token};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};
use tokio_stream::StreamExt;
use warp::http::StatusCode;

/// Request to save note
#[derive(Deserialize)]
pub struct SaveRequest {
    title: String,
    tags: String,
    content: String,
}

/// Response to save / update note
#[derive(Serialize)]
pub struct SaveNoteResponse {
    id: String,
    modified_at: i64,
    _links: NoteEndpoints,
}

/// DB note response
#[derive(Serialize)]
pub struct DBNoteResponse {
    id: String,
    modified_at: i64,
    title: String,
    tags: String,
    content: String,
}

/// Response to list notes request
#[derive(Serialize)]
pub struct ListNoteResponse {
    id: String,
    modified_at: i64,
    title: String,
    tags: String,
    content: String,
    _links: NoteEndpoints,
}

/// Save a new note
pub async fn save_note_handler(
    user_id: i32,
    note: SaveRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Saving note for user {}", user_id);

    let now = get_current_time();
    let token = get_note_token();

    let SaveRequest {
        title,
        tags,
        content,
    } = note;

    store_note(user_id, &token, now, now, &title, &tags, &content, &db).await?;

    Ok(warp::reply::json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        _links: get_note_endpoints(&token),
    }))
}

/// Delete an existing note
pub async fn delete_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Deleting note for user {}", user_id);

    let now = get_current_time();

    delete_note(user_id, &token, now, &db).await?;

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

    let now = get_current_time();

    let SaveRequest {
        title,
        tags,
        content,
    } = note;

    update_note(user_id, &token, now, &title, &tags, &content, &db).await?;

    Ok(warp::reply::json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        _links: get_note_endpoints(&token),
    }))
}

/// List all notes
pub async fn list_notes_handler(
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Listing notes for user {}", user_id);

    let notes = list_notes(user_id, &db).await?;
    let notes_with_links: Vec<ListNoteResponse> = notes
        .into_iter()
        .map(|note| ListNoteResponse {
            id: note.id.clone(),
            modified_at: note.modified_at,
            title: note.title,
            tags: note.tags,
            content: note.content,
            _links: get_note_endpoints(&note.id),
        })
        .collect();

    Ok(warp::reply::json(&notes_with_links))
}

async fn store_note(
    user_id: i32,
    token: &str,
    created_at: i64,
    modified_at: i64,
    title: &str,
    tags: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
    query!(
        "INSERT INTO notes (token, user_id, created_at, modified_at, title, tags, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7);",
        token,
        user_id,
        created_at,
        modified_at,
        title,
        tags,
        content,
    )
    .execute(db)
    .await?;

    Ok(())
}

async fn delete_note(
    user_id: i32,
    token: &str,
    deleted_at: i64,
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
    modified_at: i64,
    title: &str,
    tags: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
    let result = query!(
        "UPDATE notes
        SET modified_at = $1, title = $2, tags = $3, content = $4
        WHERE user_id = $5 AND token = $6 AND deleted_at IS NULL",
        modified_at,
        title,
        tags,
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

async fn list_notes(user_id: i32, db: &PgPool) -> Result<Vec<DBNoteResponse>, ApiError> {
    let mut rows = query!(
        "SELECT token, modified_at, title, tags, content
        FROM notes
        WHERE user_id = $1 AND deleted_at IS NULL",
        user_id
    )
    .fetch(db);

    let mut notes: Vec<DBNoteResponse> = Vec::new();

    while let Some(note) = rows.try_next().await? {
        notes.push(DBNoteResponse {
            id: note.token,
            modified_at: note.modified_at,
            title: note.title,
            tags: note.tags,
            content: note.content,
        });
    }

    Ok(notes)
}
