use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{query, PgPool};
use tokio_stream::StreamExt;

use crate::{authentication::AuthenticatedUser, error::AppError};

/// List shares response
#[derive(Serialize)]
pub struct ListShareResponse {
    token: String,
    note: String,
    view_count: i32,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// List existing shares
pub async fn list_shares_handler(
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    let shares = list_shares(user.user_id, &db).await?;

    Ok(Json(shares).into_response())
}

async fn list_shares(user_id: i32, db: &PgPool) -> Result<Vec<ListShareResponse>, AppError> {
    let mut rows = query!(
        "SELECT shares.token, shares.expires_at, notes.token AS note_token, shares.view_count, shares.created_at
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
            view_count: note.view_count,
            expires_at: note.expires_at,
            created_at: note.created_at,
        });
    }

    Ok(shares)
}
