use crate::{users::{TransactionEvent, UserCredentials, get_password, user_exists_and_matches_id, verify_password}, error::AppError, authentication::AuthorizedUser};
use axum::{Json, extract::Extension};
use chrono::Utc;
use hyper::StatusCode;
use sqlx::{query, PgPool};

/// Delete user with all associated data
pub async fn delete_user_handler(
    Json(credentials): Json<UserCredentials>,
    user: AuthorizedUser,
    db: Extension<PgPool>,
) -> Result<StatusCode, AppError> {
    if !user_exists_and_matches_id(&credentials.name, user.user_id, &db).await? {
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let password = get_password(&credentials.name, &db).await?;

    match verify_password(&credentials.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED),
        true => (),
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

    let now = Utc::now();

    query!(
        "INSERT INTO transactions (user_id, event, date)
        VALUES ($1, $2, $3);",
        user_id,
        TransactionEvent::PauseFieldnotes as TransactionEvent,
        now,
    )
    .execute(&mut tx)
    .await?;

    query!(
        "UPDATE users
        SET deleted_at = $1
        WHERE id = $2;",
        now,
        user_id
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
