use serde::Deserialize;
use sqlx::{query, PgPool};

/// This request form is expected for storing salt
#[derive(Deserialize)]
pub struct UserSaltRequest {
    salt: String,
}

/// Store user salt
pub async fn store_salt_handler(
    user_id: i32,
    salt: UserSaltRequest,
    db: PgPool,
) -> Result<impl warp::Reply, ApiError> {
    store_salt(user_id, &salt.salt, &db).await?;

    Ok(StatusCode::OK)
}

// Update password of existing user
async fn store_salt(user_id: i32, salt: &str, db: &PgPool) -> Result<(), ApiError> {
    let result = query!(
        "UPDATE users 
        SET salt = $1
        WHERE id = $2",
        salt,
        user_id,
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 1 {
        Ok(())
    } else {
        Err(ApiError::ViolatedAssertion(
            "No rows affected when storing salt".to_string(),
        ))
    }
}
