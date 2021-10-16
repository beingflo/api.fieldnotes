use crate::error::ApiError;
use chrono::{DateTime, Utc};
use log::info;
use sqlx::{PgPool, query};
use warp::{http::StatusCode};

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
