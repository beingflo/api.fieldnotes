use crate::error::ApiError;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{query, PgPool};
use tokio_stream::StreamExt;

/// List publications response
#[derive(Serialize)]
pub struct ListPublicationResponse {
    token: String,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
    metadata: String,
    public: String,
    key: String,
}

/// List published notes
pub async fn list_publications_handler(
    username: String,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
    let shares = list_publications(username, &db).await?;

    Ok(warp::reply::json(&shares))
}

async fn list_publications(
    username: String,
    db: &PgPool,
) -> Result<Vec<ListPublicationResponse>, ApiError> {
    let mut rows = query!(
        "SELECT shares.token, notes.created_at, notes.modified_at, notes.metadata, notes.key, shares.public
        FROM shares 
        INNER JOIN notes ON shares.note_id = notes.id
        INNER JOIN users ON notes.user_id = users.id
        WHERE users.username = $1 AND shares.public IS NOT NULL",
        username
    )
    .fetch(db);

    let mut publications: Vec<ListPublicationResponse> = Vec::new();

    while let Some(row) = rows.try_next().await? {
        publications.push(ListPublicationResponse {
            token: row.token,
            created_at: row.created_at,
            modified_at: row.modified_at,
            metadata: row.metadata,
            key: row.key,
            public: row.public.unwrap(),
        });
    }

    Ok(publications)
}
