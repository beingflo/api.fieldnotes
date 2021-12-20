use crate::error_warp::ApiError;
use chrono::{DateTime, Utc};
use sqlx::{query, PgPool};
use warp::http::StatusCode;

/// Delete an existing note
pub async fn delete_note_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
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
    let mut tx = db.begin().await?;

    let result = query!(
        "UPDATE notes
        SET deleted_at = $1
        WHERE user_id = $2 AND token = $3 AND deleted_at IS NULL",
        deleted_at,
        user_id,
        token,
    )
    .execute(&mut tx)
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
    .execute(&mut tx)
    .await?;

    if result.rows_affected() == 1 {
        tx.commit().await?;
        Ok(())
    } else {
        tx.rollback().await?;
        Err(ApiError::Unauthorized)
    }
}
