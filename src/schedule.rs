use crate::authentication::TOKEN_EXPIRATION_WEEKS;
use chrono::{Duration, Utc};
use log::{error, info};
use sqlx::{query, PgPool};
use tokio::time::{interval_at, Instant};

pub async fn notes_deletion_schedule(db: PgPool) {
    let mut interval_timer = interval_at(
        Instant::now() + Duration::minutes(7).to_std().unwrap(),
        Duration::hours(7).to_std().unwrap(),
    );
    loop {
        interval_timer.tick().await;

        let db_clone = db.clone();

        tokio::spawn(async move {
            delete_expired_notes(&db_clone).await;
        });
    }
}

async fn delete_expired_notes(db: &PgPool) {
    let one_month_ago = Utc::now() - Duration::weeks(4);
    match query!(
        "DELETE
        FROM notes 
        WHERE deleted_at IS NOT NULL AND deleted_at < $1;",
        one_month_ago,
    )
    .execute(db)
    .await
    {
        Ok(result) => {
            info!(
                "Deletion of expired notes with {} affected items",
                result.rows_affected()
            )
        }
        Err(error) => {
            error!("Deletion of expired notes caused error: {}", error)
        }
    };
}

pub async fn tokens_deletion_schedule(db: PgPool) {
    let mut interval_timer = interval_at(
        Instant::now() + Duration::minutes(3).to_std().unwrap(),
        Duration::hours(3).to_std().unwrap(),
    );
    loop {
        interval_timer.tick().await;

        let db_clone = db.clone();

        tokio::spawn(async move {
            delete_expired_tokens(&db_clone).await;
        });
    }
}

async fn delete_expired_tokens(db: &PgPool) {
    let token_expiration_period = Utc::now() - Duration::weeks(TOKEN_EXPIRATION_WEEKS);
    match query!(
        "DELETE
        FROM auth_tokens
        WHERE created_at < $1;",
        token_expiration_period,
    )
    .execute(db)
    .await
    {
        Ok(result) => {
            info!(
                "Deletion of expired auth tokens with {} affected items",
                result.rows_affected()
            )
        }
        Err(error) => {
            error!("Deletion of expired auth tokens caused error: {}", error)
        }
    };
}
