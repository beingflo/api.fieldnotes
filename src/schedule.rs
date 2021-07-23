use chrono::{Duration, Utc};
use log::{error, info};
use sqlx::{query, PgPool};
use tokio::time::{interval_at, Instant};

const DAILY_BALANCE_DECREASE: i64 = 16_438;

// Discount balances of all user accounts by the appropriate amount.
// Should run once a day.
pub async fn decrease_balances(db: &PgPool) {
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
            error!("Balance decrease error: {}", error)
        }
    };
}

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
