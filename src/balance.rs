use log::{error, info};
use sqlx::{query, PgPool};

/// Balance is stored as CHF * 10^6 to avoid significant rounding errors

/// Default starting balance for new users
/// 0.5 CHF = 500'000
pub const DEFAULT_BALANCE: i64 = 500_000;
pub const DAILY_BALANCE_DECREASE: i64 = 16_438;

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
    let mut interval_timer = tokio::time::interval(chrono::Duration::days(1).to_std().unwrap());
    loop {
        interval_timer.tick().await;

        let db_clone = db.clone();

        tokio::spawn(async move {
            decrease_balances(&db_clone).await;
        });
    }
}
