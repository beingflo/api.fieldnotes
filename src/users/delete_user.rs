use crate::error::ApiError;
use crate::users::{get_password, user_exists_and_matches_id, verify_password, UserCredentials};
use chrono::Utc;
use log::{info, warn};
use sqlx::{query, PgPool};
use warp::http::StatusCode;

/// Delete user with all associated data
pub async fn delete_user(
    credentials: UserCredentials,
    user_id: i32,
    db: PgPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Deleting user {}", credentials.name);

    if !user_exists_and_matches_id(&credentials.name, user_id, &db).await? {
        warn!(
            "User {} doesn't exists or doesn't match auth token",
            user_id
        );
        return Ok(StatusCode::UNAUTHORIZED);
    }

    let password = get_password(&credentials.name, &db).await?;

    match verify_password(&credentials.name, &credentials.password, &password).await? {
        false => return Ok(StatusCode::UNAUTHORIZED),
        true => (),
    }

    delete_all_user_data(user_id, &db).await?;

    Ok(StatusCode::OK)
}

/// Delete all user data
pub async fn delete_all_user_data(user_id: i32, db: &PgPool) -> Result<(), ApiError> {
    let mut tx = db.begin().await?;

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
