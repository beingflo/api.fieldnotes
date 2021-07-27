use crate::error::ApiError;
use crate::util::get_share_token;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};
use tokio_stream::StreamExt;

/// Request to create share
#[derive(Deserialize)]
pub struct CreateShareRequest {
    note: String,
}

/// Request to create share
#[derive(Serialize)]
pub struct CreateShareResponse {
    token: String,
}

/// List shares response
#[derive(Serialize)]
pub struct ListShareResponse {
    token: String,
    note_token: String,
    created_at: DateTime<Utc>,
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

    create_share(&token, &request.note, user_id, now, &db).await?;

    Ok(warp::reply::json(&CreateShareResponse { token: token }))
}

async fn create_share(
    token: &str,
    note: &str,
    user_id: i32,
    created_at: DateTime<Utc>,
    db: &PgPool,
) -> Result<(), ApiError> {
    let row = query!(
        "INSERT INTO shares (token, note_id, user_id, created_at)
        SELECT $1, id, $3, $4
        FROM notes WHERE token = $2 AND user_id = $3 AND deleted_at IS NULL;",
        token,
        note,
        user_id,
        created_at
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

/// List existing shares
pub async fn list_shares_handler(
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Listing shares for user {}", user_id);

    let shares = list_shares(user_id, &db).await?;

    Ok(warp::reply::json(&shares))
}

async fn list_shares(user_id: i32, db: &PgPool) -> Result<Vec<ListShareResponse>, ApiError> {
    let mut rows = query!(
        "SELECT shares.token, notes.token AS note_token, shares.created_at
        FROM shares 
        INNER JOIN notes ON shares.note_id = notes.id
        WHERE shares.user_id = $1;",
        user_id
    )
    .fetch(db);

    let mut shares: Vec<ListShareResponse> = Vec::new();

    while let Some(note) = rows.try_next().await? {
        shares.push(ListShareResponse {
            token: note.token,
            note_token: note.note_token,
            created_at: note.created_at,
        });
    }

    Ok(shares)
}
