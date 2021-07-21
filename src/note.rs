use crate::endpoint::{get_note_endpoints, NoteEndpoints};
use crate::error::ApiError;
use crate::util::get_note_token;
use chrono::{DateTime, Utc};
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
    modified_at: DateTime<Utc>,
    _links: NoteEndpoints,
}

/// DB list note response
#[derive(Serialize)]
pub struct DBListNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    title: String,
    tags: String,
}

/// Response to list notes request
#[derive(Serialize)]
pub struct ListNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    title: String,
    tags: String,
    _links: NoteEndpoints,
}

/// DB get note response
#[derive(Serialize)]
pub struct DBGetNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    title: String,
    tags: String,
    content: String,
}

/// Response to get note request
#[derive(Serialize)]
pub struct GetNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
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

    let now = Utc::now();
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

/// Get an existing note
pub async fn get_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Get note for user {}", user_id);

    let note = get_note(user_id, &token, &db).await?;

    Ok(warp::reply::json(&GetNoteResponse {
        id: note.id,
        modified_at: note.modified_at,
        title: note.title,
        tags: note.tags,
        content: note.content,
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
            _links: get_note_endpoints(&note.id),
        })
        .collect();

    Ok(warp::reply::json(&notes_with_links))
}

async fn get_note(user_id: i32, token: &str, db: &PgPool) -> Result<DBGetNoteResponse, ApiError> {
    match query!(
        "SELECT token, modified_at, title, tags, content
        FROM notes
        WHERE user_id = $1 AND token = $2 AND deleted_at IS NULL",
        user_id,
        token,
    )
    .fetch_optional(db)
    .await?
    {
        Some(row) => Ok(DBGetNoteResponse {
            id: token.to_string(),
            modified_at: row.modified_at,
            title: row.title,
            tags: row.tags,
            content: row.content,
        }),
        None => {
            return Err(ApiError::Unauthorized);
        }
    }
}

async fn store_note(
    user_id: i32,
    token: &str,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
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

async fn list_notes(user_id: i32, db: &PgPool) -> Result<Vec<DBListNoteResponse>, ApiError> {
    let mut rows = query!(
        "SELECT token, modified_at, title, tags
        FROM notes
        WHERE user_id = $1 AND deleted_at IS NULL",
        user_id
    )
    .fetch(db);

    let mut notes: Vec<DBListNoteResponse> = Vec::new();

    while let Some(note) = rows.try_next().await? {
        notes.push(DBListNoteResponse {
            id: note.token,
            modified_at: note.modified_at,
            title: note.title,
            tags: note.tags,
        });
    }

    Ok(notes)
}
