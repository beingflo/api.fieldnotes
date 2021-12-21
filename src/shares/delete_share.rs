use axum::{
    extract::{Extension, Path},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use sqlx::{query, PgPool};

use crate::{authentication::AuthenticatedUser, error::AppError};

pub async fn delete_share_handler(
    Path(token): Path<String>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<Response, AppError> {
    delete_share(user.user_id, &token, &db).await?;

    Ok(StatusCode::OK.into_response())
}

async fn delete_share(user_id: i32, token: &str, db: &PgPool) -> Result<(), AppError> {
    let row = query!(
        "DELETE
        FROM shares
        WHERE token = $1 AND user_id = $2;",
        token,
        user_id,
    )
    .execute(db)
    .await?;

    if row.rows_affected() == 1 {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}
