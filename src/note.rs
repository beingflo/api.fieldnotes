use crate::endpoint::{get_note_endpoints, NoteEndpoints};
use crate::error::ApiError;
use crate::util::{get_current_time, get_note_token};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

/// Request to save note.
#[derive(Deserialize)]
pub struct SaveRequest {
    title: String,
    tags: String,
    content: String,
}

/// Response to save / update note.
#[derive(Serialize)]
pub struct SaveNoteResponse {
    id: String,
    created_at: i64,
    modified_at: i64,
    _links: NoteEndpoints,
}

/// Save a note to db.
pub async fn save_note(
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
        created_at: now,
        modified_at: now,
        _links: get_note_endpoints(&token),
    }))
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
