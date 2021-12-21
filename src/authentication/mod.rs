use crate::{
    error::AppError,
    users::is_funded,
    util::{get_token_from_header, truncate_auth_token},
};
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
};
use chrono::{DateTime, Duration, Utc};
use log::info;
use sqlx::{query, PgPool, Pool, Postgres};

/// Token expiration time: 2 months
pub const TOKEN_EXPIRATION_WEEKS: i64 = 8;

pub struct AuthenticatedFundedUser {
    pub user_id: i32,
    pub auth_token: String,
    pub username: String,
}

#[async_trait]
impl<B> FromRequest<B> for AuthenticatedFundedUser
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(db) = Extension::<Pool<Postgres>>::from_request(req)
            .await
            .expect("db missing");

        let token = get_token_from_header(req.headers().expect("Header unavailable"))?;

        let user = is_authorized_with_user(token, &db).await?;

        is_funded(user.user_id, &db).await?;

        Ok(AuthenticatedFundedUser {
            user_id: user.user_id,
            auth_token: user.auth_token,
            username: user.username,
        })
    }
}
pub struct AuthenticatedUser {
    pub user_id: i32,
    pub auth_token: String,
    pub username: String,
}

#[async_trait]
impl<B> FromRequest<B> for AuthenticatedUser
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(db) = Extension::<Pool<Postgres>>::from_request(req)
            .await
            .expect("db missing");

        let token = get_token_from_header(req.headers().expect("Header unavailable"))?;

        let user = is_authorized_with_user(token, &db).await?;

        Ok(user)
    }
}

// Checks if user has proper authorization token for request and return user id
// used in further filters and handlers.
pub async fn is_authorized_with_user(
    token: String,
    db: &PgPool,
) -> Result<AuthenticatedUser, AppError> {
    let (authorized_user, created_at) = get_auth_token_info(&token, &db).await?;

    let now = Utc::now();

    info!("Access with token: {}", truncate_auth_token(&token));

    if created_at + Duration::weeks(TOKEN_EXPIRATION_WEEKS) > now {
        Ok(authorized_user)
    } else {
        Err(AppError::Unauthorized)
    }
}

// Get user_id and creation date of provided token
async fn get_auth_token_info(
    token: &str,
    db: &PgPool,
) -> Result<(AuthenticatedUser, DateTime<Utc>), AppError> {
    match query!(
        "SELECT users.id, users.username, auth_tokens.token, auth_tokens.created_at
        FROM auth_tokens 
        INNER JOIN users ON users.id = auth_tokens.user_id
        WHERE auth_tokens.token = $1;",
        token
    )
    .fetch_optional(db)
    .await?
    {
        Some(tok) => Ok((
            AuthenticatedUser {
                user_id: tok.id,
                auth_token: tok.token,
                username: tok.username,
            },
            tok.created_at,
        )),
        None => Err(AppError::Unauthorized),
    }
}

/// Add a new token to the user. User is expected to exist.
pub async fn store_auth_token(
    name: &str,
    token: &str,
    created_at: DateTime<Utc>,
    db: &PgPool,
) -> Result<(), AppError> {
    query!(
        "INSERT INTO auth_tokens (token, created_at, user_id)
        VALUES ($1, $2, (SELECT id FROM users WHERE username=$3));",
        token,
        created_at,
        name
    )
    .execute(db)
    .await?;

    Ok(())
}

// Delete provided auth token from db
pub async fn delete_auth_token(token: &str, db: &PgPool) -> Result<(), AppError> {
    query!(
        "DELETE
        FROM auth_tokens 
        WHERE token = $1",
        &token
    )
    .execute(db)
    .await?;

    Ok(())
}

// Delete all auth tokens of user from db
pub async fn delete_all_auth_tokens(user_id: i32, db: &PgPool) -> Result<(), AppError> {
    query!(
        "DELETE
        FROM auth_tokens 
        WHERE user_id = $1",
        user_id
    )
    .execute(db)
    .await?;

    Ok(())
}
