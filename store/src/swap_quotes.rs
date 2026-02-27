use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;
use crate::{dto::SwapQuoteRow, pool::pool};

pub async fn insert_swap_quote(
    user_id: Uuid,
    input_mint: &str,
    output_mint: &str,
    quoted_amount: i64,
    slippage_bps: i32,
    price_impact_pct: Option<f64>,
    route_plan: Value,
    unsigned_tx: &str,
    expires_at: DateTime<Utc>,
) -> Result<SwapQuoteRow, sqlx::Error> {
    sqlx::query_as!(
        SwapQuoteRow,
        r#"
        INSERT INTO swap_quotes
          (user_id, input_mint, output_mint, quoted_amount, slippage_bps,
           price_impact_pct, route_plan, unsigned_tx, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
          id, user_id, input_mint, output_mint, quoted_amount, slippage_bps,
          CAST(price_impact_pct AS FLOAT8) AS price_impact_pct,
          route_plan AS "route_plan: Value",
          unsigned_tx, expires_at, status, transaction_id, created_at
        "#,
        user_id,
        input_mint,
        output_mint,
        quoted_amount,
        slippage_bps,
        price_impact_pct as Option<f64>,
        route_plan,
        unsigned_tx,
        expires_at,
    )
    .fetch_one(pool())
    .await
}

pub async fn find_swap_quote_by_id(
    id: Uuid,
) -> Result<Option<SwapQuoteRow>, sqlx::Error> {
    sqlx::query_as!(
        SwapQuoteRow,
        r#"
        SELECT
          id, user_id, input_mint, output_mint, quoted_amount, slippage_bps,
          CAST(price_impact_pct AS FLOAT8) AS price_impact_pct,
          route_plan AS "route_plan: Value",
          unsigned_tx, expires_at, status, transaction_id, created_at
        FROM swap_quotes
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool())
    .await
}

pub async fn update_swap_quote_status(
    id: Uuid,
    status: &str,
    transaction_id: Option<Uuid>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE swap_quotes SET status = $1, transaction_id = $2 WHERE id = $3",
        status,
        transaction_id,
        id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn find_pending_quotes_by_user(
    user_id: Uuid,
) -> Result<Vec<SwapQuoteRow>, sqlx::Error> {
    sqlx::query_as!(
        SwapQuoteRow,
        r#"
        SELECT
          id, user_id, input_mint, output_mint, quoted_amount, slippage_bps,
          CAST(price_impact_pct AS FLOAT8) AS price_impact_pct,
          route_plan AS "route_plan: Value",
          unsigned_tx, expires_at, status, transaction_id, created_at
        FROM swap_quotes
        WHERE user_id = $1 AND status = 'pending' AND expires_at > NOW()
        ORDER BY created_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool())
    .await
}

pub async fn expire_stale_quotes() -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE swap_quotes SET status = 'expired' WHERE status = 'pending' AND expires_at <= NOW()"
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected())
}
