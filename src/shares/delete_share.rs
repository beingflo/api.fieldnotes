use crate::error::ApiError;
use log::info;
use sqlx::{query, PgPool};
use warp::http::StatusCode;

pub async fn delete_share_handler(
    token: String,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Deleting share for user {}", user_id);

    delete_share(user_id, &token, &db).await?;

    Ok(StatusCode::OK)
}

async fn delete_share(user_id: i32, token: &str, db: &PgPool) -> Result<(), ApiError> {
    let row = query!(
        "DELETE
        FROM shares
        WHERE token = $1 AND user_id = $2;",
        token,
        user_id,
    )
    .execute(db)
    .await?;

    match row.rows_affected() {
        0 => Err(ApiError::Unauthorized),
        1 => Ok(()),
        _ => Err(ApiError::ViolatedAssertion(
            "Deleting share affected multiple rows".to_string(),
        )),
    }
}
