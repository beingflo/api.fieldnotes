use crate::error::ApiError;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query};

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
    token: String,
    user_id: i32,
    note: UpdateNoteRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Updating note for user {}", user_id);

    let now = Utc::now();

    let UpdateNoteRequest {
        metadata,
        key,
        content,
    } = note;

    update_note(
        user_id,
        &token,
        now,
        &metadata,
        &key,
        &content,
        &db,
    )
    .await?;

    Ok(warp::reply::json(&UpdateNoteResponse {
        id: token.clone(),
        modified_at: now,
    }))
}

async fn update_note(
    user_id: i32,
    token: &str,
    modified_at: DateTime<Utc>,
    metadata: &str,
    key: &str,
    content: &str,
    db: &PgPool,
) -> Result<(), ApiError> {
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
    } else if result.rows_affected() == 0 {
        Err(ApiError::Unauthorized)
    } else {
        Err(ApiError::ViolatedAssertion(
            "Multiple rows affected when updating note".to_string(),
        ))
    }
}
