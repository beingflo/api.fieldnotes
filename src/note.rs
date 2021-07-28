use crate::endpoint::{get_note_endpoints, NoteEndpoints};
use crate::error::ApiError;
use crate::util::get_note_token;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};
use std::collections::HashMap;
use tokio_stream::StreamExt;
use warp::http::StatusCode;

/// Request to save note
#[derive(Deserialize)]
pub struct SaveRequest {
    metainfo: String,
    encrypted_key: String,
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
    metainfo: String,
    encrypted_key: String,
}

/// Response to list notes request
#[derive(Serialize)]
pub struct ListNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    metainfo: String,
    encrypted_key: String,
    _links: NoteEndpoints,
}

/// DB list deleted note response
#[derive(Serialize)]
pub struct DBListDeletedNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    deleted_at: DateTime<Utc>,
    metainfo: String,
    encrypted_key: String,
}

/// Response to list notes request
#[derive(Serialize)]
pub struct ListDeletedNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    deleted_at: DateTime<Utc>,
    metainfo: String,
    encrypted_key: String,
    _links: NoteEndpoints,
}

/// DB get note response
#[derive(Serialize)]
pub struct DBGetNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    metainfo: String,
    encrypted_key: String,
    content: String,
}

/// Response to get note request
#[derive(Serialize)]
pub struct GetNoteResponse {
    id: String,
    modified_at: DateTime<Utc>,
    metainfo: String,
    encrypted_key: String,
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
        metainfo,
        encrypted_key,
        content,
    } = note;

    store_note(
        user_id,
        &token,
        now,
        now,
        &metainfo,
        &encrypted_key,
        &content,
        &db,
    )
    .await?;

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
        metainfo: note.metainfo,
        encrypted_key: note.encrypted_key,
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
        metainfo,
        encrypted_key,
        content,
    } = note;

    update_note(
        user_id,
        &token,
        now,
        &metainfo,
        &encrypted_key,
        &content,
        &db,
    )
    .await?;

    Ok(warp::reply::json(&SaveNoteResponse {
        id: token.clone(),
        modified_at: now,
        _links: get_note_endpoints(&token),
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
        let notes_with_links: Vec<ListDeletedNoteResponse> = notes
            .into_iter()
            .map(|note| ListDeletedNoteResponse {
                id: note.id.clone(),
                modified_at: note.modified_at,
                deleted_at: note.deleted_at,
                metainfo: note.metainfo,
                encrypted_key: note.encrypted_key,
                _links: get_note_endpoints(&note.id),
            })
            .collect();

        Ok(warp::reply::json(&notes_with_links))
    } else {
        info!("Listing notes for user {}", user_id);

        let notes = list_notes(user_id, &db).await?;
        let notes_with_links: Vec<ListNoteResponse> = notes
            .into_iter()
            .map(|note| ListNoteResponse {
                id: note.id.clone(),
                modified_at: note.modified_at,
                metainfo: note.metainfo,
                encrypted_key: note.encrypted_key,
                _links: get_note_endpoints(&note.id),
            })
            .collect();

        Ok(warp::reply::json(&notes_with_links))
    }
}

async fn get_note(user_id: i32, token: &str, db: &PgPool) -> Result<DBGetNoteResponse, ApiError> {
    match query!(
        "SELECT token, modified_at, metainfo, encrypted_key, content
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
            metainfo: row.metainfo,
            encrypted_key: row.encrypted_key,
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
    metainfo: &str,
    encrypted_key: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
    query!(
        "INSERT INTO notes (token, user_id, created_at, modified_at, metainfo, encrypted_key, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7);",
        token,
        user_id,
        created_at,
        modified_at,
        metainfo,
        encrypted_key,
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
    metainfo: &str,
    encrypted_key: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
    let result = query!(
        "UPDATE notes
        SET modified_at = $1, metainfo = $2, encrypted_key = $3, content = $4
        WHERE user_id = $5 AND token = $6 AND deleted_at IS NULL",
        modified_at,
        metainfo,
        encrypted_key,
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
        "SELECT token, modified_at, metainfo, encrypted_key 
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
            metainfo: note.metainfo,
            encrypted_key: note.encrypted_key,
        });
    }

    Ok(notes)
}

async fn list_deleted_notes(
    user_id: i32,
    db: &PgPool,
) -> Result<Vec<DBListDeletedNoteResponse>, ApiError> {
    let mut rows = query!(
        "SELECT token, modified_at, deleted_at, metainfo, encrypted_key 
        FROM notes
        WHERE user_id = $1 AND deleted_at IS NOT NULL",
        user_id
    )
    .fetch(db);

    let mut notes: Vec<DBListDeletedNoteResponse> = Vec::new();

    while let Some(note) = rows.try_next().await? {
        notes.push(DBListDeletedNoteResponse {
            id: note.token,
            modified_at: note.modified_at,
            deleted_at: note.deleted_at.unwrap(),
            metainfo: note.metainfo,
            encrypted_key: note.encrypted_key,
        });
    }

    Ok(notes)
}
