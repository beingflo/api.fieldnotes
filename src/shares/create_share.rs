use crate::error::ApiError;
use crate::util::get_share_token;
use chrono::{DateTime, Duration, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

/// Request to create share
#[derive(Deserialize)]
pub struct CreateShareRequest {
    note: String,
    public: Option<String>,
    expires_in: Option<i64>,
}

/// Request to create share
#[derive(Serialize)]
pub struct CreateShareResponse {
    token: String,
    note: String,
    public: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// Create a new share from an existing note
pub async fn create_share_handler(
    user_id: i32,
    request: CreateShareRequest,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Creating share for user {}", user_id);

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
        return Err(ApiError::Conflict.into());
    }
    create_share(
        &token,
        &request.note,
        user_id,
        &request.public,
        now,
        expires_at,
        &db,
    )
    .await?;

    Ok(warp::reply::json(&CreateShareResponse {
        token,
        created_at: now,
        expires_at,
        note: request.note,
        public: request.public,
    }))
}

async fn create_share(
    token: &str,
    note: &str,
    user_id: i32,
    public: &Option<String>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    db: &PgPool,
) -> Result<(), ApiError> {
    let row = query!(
        "INSERT INTO shares (token, note_id, user_id, created_at, expires_at, public)
        SELECT $1, id, $3, $4, $5, $6
        FROM notes WHERE token = $2 AND user_id = $3 AND deleted_at IS NULL",
        token,
        note,
        user_id,
        created_at,
        expires_at,
        *public,
    )
    .execute(db)
    .await?;

    match row.rows_affected() {
        0 => Err(ApiError::Unauthorized),
        1 => Ok(()),
        _ => Err(ApiError::ViolatedAssertion(
            "Creating share affected multiple rows".to_string(),
        )),
    }
}

async fn share_exists(note_token: &str, db: &PgPool) -> Result<bool, ApiError> {
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
