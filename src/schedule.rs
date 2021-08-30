use crate::authentication::TOKEN_EXPIRATION_WEEKS;
use chrono::{Duration, Utc};
use log::{error, info};
use sqlx::{query, PgPool};
use tokio::time::{interval_at, Instant};

const DAILY_BALANCE_DECREASE: i64 = 32_876;

pub async fn balance_decrease_schedule(db: PgPool) {
    let midnight = {
        let now = Utc::now();

        let tomorrow_midnight = (now + Duration::days(1)).date().and_hms(0, 0, 0);

        tomorrow_midnight
            .signed_duration_since(now)
            .to_std()
            .unwrap()
    };

    let mut interval_timer = interval_at(
        Instant::now() + midnight,
        Duration::days(1).to_std().unwrap(),
    );
    loop {
        interval_timer.tick().await;

        let db_clone = db.clone();

        tokio::spawn(async move {
            decrease_balances(&db_clone).await;
        });
    }
}

// Discount balances of all user accounts by the appropriate amount.
// Should run once a day.
async fn decrease_balances(db: &PgPool) {
    match query!(
        "UPDATE users 
        SET balance = balance - $1
        WHERE deleted_at IS NULL;",
        DAILY_BALANCE_DECREASE,
    )
    .execute(db)
    .await
    {
        Ok(result) => {
            info!(
                "Balance decrease executed with {} affected rows",
                result.rows_affected()
            )
        }
        Err(error) => {
            error!("Balance decrease caused error: {}", error)
        }
    };
}

pub async fn notes_deletion_schedule(db: PgPool) {
    let mut interval_timer = interval_at(
        Instant::now() + Duration::minutes(5).to_std().unwrap(),
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
                "Deletion of expired notes with {} affected rows",
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
                "Deletion of expired auth tokens with {} affected rows",
                result.rows_affected()
            )
        }
        Err(error) => {
            error!("Deletion of expired auth tokens caused error: {}", error)
        }
    };
}
