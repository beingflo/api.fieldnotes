use crate::{authentication::AuthenticatedUser, error::AppError, users::UserCredentials};
use axum::{extract::Extension, Json};
use hyper::StatusCode;
use sqlx::{query, PgPool};

use super::validate_user_with_credentials;

/// Delete user with all associated data
pub async fn delete_user_handler(
    Json(credentials): Json<UserCredentials>,
    user: AuthenticatedUser,
    db: Extension<PgPool>,
) -> Result<StatusCode, AppError> {
    if !validate_user_with_credentials(
        &user.username,
        user.user_id,
        &credentials.name,
        &credentials.password,
        &db,
    )
    .await?
    {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    delete_all_user_data(user.user_id, &db).await?;

    Ok(StatusCode::OK)
}

/// Delete all user data
pub async fn delete_all_user_data(user_id: i32, db: &PgPool) -> Result<(), AppError> {
    let mut tx = db.begin().await?;

    query!(
        "DELETE
        FROM shares 
        WHERE user_id = $1;",
        user_id
    )
    .execute(&mut tx)
    .await?;

    query!(
        "DELETE
        FROM notes
        WHERE user_id = $1;",
        user_id
    )
    .execute(&mut tx)
    .await?;

    query!(
        "DELETE
        FROM auth_tokens 
        WHERE user_id = $1;",
        user_id
    )
    .execute(&mut tx)
    .await?;

    query!(
        "DELETE
        FROM users
        WHERE id = $1;",
        user_id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
