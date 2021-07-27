use crate::error::ApiError;
use crate::util::get_share_token;
use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{query, PgPool};

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
        "INSERT INTO shares (token, note_id, created_at)
        SELECT $1, id, $3
        FROM notes WHERE token = $2 AND user_id = $4 AND deleted_at IS NULL;",
        token,
        note,
        created_at,
        user_id
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
