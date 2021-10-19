use crate::error::ApiError;
use chrono::{DateTime, Utc};
use log::info;
use serde::Serialize;
use sqlx::{query, PgPool};
use tokio_stream::StreamExt;

/// List shares response
#[derive(Serialize)]
pub struct ListShareResponse {
    token: String,
    note: String,
    public: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// List existing shares
pub async fn list_shares_handler(user_id: i32, db: PgPool) -> Result<impl warp::Reply, ApiError> {
    info!("Listing shares for user {}", user_id);

    let shares = list_shares(user_id, &db).await?;

    Ok(warp::reply::json(&shares))
}

async fn list_shares(user_id: i32, db: &PgPool) -> Result<Vec<ListShareResponse>, ApiError> {
    let mut rows = query!(
        "SELECT shares.token, shares.expires_at, notes.token AS note_token, shares.created_at, shares.public
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
            note: note.note_token,
            expires_at: note.expires_at,
            created_at: note.created_at,
            public: note.public,
        });
    }

    Ok(shares)
}
