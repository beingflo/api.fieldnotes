use crate::{authentication::AuthenticatedFundedUser, error::AppError, util::get_share_token};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

/// Request to create share
#[derive(Deserialize)]
pub struct CreateShareRequest {
    note: String,
    expires_in: Option<i64>,
}

/// Request to create share
#[derive(Serialize)]
pub struct CreateShareResponse {
    token: String,
    note: String,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// Create a new share from an existing note
pub async fn create_share_handler(
    user: AuthenticatedFundedUser,
    Json(request): Json<CreateShareRequest>,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    let now = Utc::now();
    let token = get_share_token();

    let expires_at = {
        let hours = request.expires_in;
        if hours.is_none() {
            None
        } else {
            Some(now + Duration::hours(hours.unwrap()))
        }
    };

    if share_exists(&request.note, &db).await? {
        return Err(AppError::Conflict);
    }

    create_share(&token, &request.note, user.user_id, now, expires_at, &db).await?;

    Ok(Json(&CreateShareResponse {
        token,
        created_at: now,
        expires_at,
        note: request.note,
    })
    .into_response())
}

async fn create_share(
    token: &str,
    note: &str,
    user_id: i32,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    db: &PgPool,
) -> Result<(), AppError> {
    let row = query!(
        "INSERT INTO shares (token, note_id, user_id, created_at, expires_at, view_count)
        SELECT $1, id, $3, $4, $5, $6
        FROM notes WHERE token = $2 AND user_id = $3 AND deleted_at IS NULL",
        token,
        note,
        user_id,
        created_at,
        expires_at,
        0,
    )
    .execute(db)
    .await?;

    if row.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

async fn share_exists(note_token: &str, db: &PgPool) -> Result<bool, AppError> {
    match query!(
        "SELECT shares.id
        FROM shares 
        WHERE shares.note_id = (
            SELECT notes.id
            FROM notes 
            WHERE notes.token = $1
        );",
        note_token
    )
    .fetch_optional(db)
    .await?
    {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}
