use uuid::Uuid;
use crate::{dto::WalletBalanceRow, pool::pool};

pub async fn upsert_balance(
    user_id: Uuid,
    wallet_pubkey: &str,
    mint: &str,
    symbol: &str,
    decimals: i32,
    raw_amount: i64,
    ui_amount: Option<f64>,
    usd_value: Option<f64>,
) -> Result<WalletBalanceRow, sqlx::Error> {
    sqlx::query_as!(
        WalletBalanceRow,
        r#"
        INSERT INTO wallet_balances
          (user_id, wallet_pubkey, mint, symbol, decimals, raw_amount, ui_amount, usd_value, last_synced_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        ON CONFLICT (wallet_pubkey, mint) DO UPDATE
          SET raw_amount     = EXCLUDED.raw_amount,
              ui_amount      = EXCLUDED.ui_amount,
              usd_value      = EXCLUDED.usd_value,
              last_synced_at = NOW()
        RETURNING
          id, user_id, wallet_pubkey, mint, symbol, decimals, raw_amount,
          CAST(ui_amount  AS FLOAT8) AS ui_amount,
          CAST(usd_value  AS FLOAT8) AS usd_value,
          last_synced_at
        "#,
        user_id,
        wallet_pubkey,
        mint,
        symbol,
        decimals,
        raw_amount,
        ui_amount as Option<f64>,
        usd_value as Option<f64>,
    )
    .fetch_one(pool())
    .await
}

pub async fn find_balances_by_user(
    user_id: Uuid,
) -> Result<Vec<WalletBalanceRow>, sqlx::Error> {
    sqlx::query_as!(
        WalletBalanceRow,
        r#"
        SELECT
          id, user_id, wallet_pubkey, mint, symbol, decimals, raw_amount,
          CAST(ui_amount  AS FLOAT8) AS ui_amount,
          CAST(usd_value  AS FLOAT8) AS usd_value,
          last_synced_at
        FROM wallet_balances
        WHERE user_id = $1
        ORDER BY usd_value DESC NULLS LAST
        "#,
        user_id,
    )
    .fetch_all(pool())
    .await
}

pub async fn find_balance_by_pubkey_and_mint(
    wallet_pubkey: &str,
    mint: &str,
) -> Result<Option<WalletBalanceRow>, sqlx::Error> {
    sqlx::query_as!(
        WalletBalanceRow,
        r#"
        SELECT
          id, user_id, wallet_pubkey, mint, symbol, decimals, raw_amount,
          CAST(ui_amount  AS FLOAT8) AS ui_amount,
          CAST(usd_value  AS FLOAT8) AS usd_value,
          last_synced_at
        FROM wallet_balances
        WHERE wallet_pubkey = $1 AND mint = $2
        "#,
        wallet_pubkey,
        mint,
    )
    .fetch_optional(pool())
    .await
}

pub async fn update_usd_value(
    id: Uuid,
    usd_value: f64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE wallet_balances SET usd_value = $1, last_synced_at = NOW() WHERE id = $2",
        usd_value as f64,
        id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_balances_by_user(user_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM wallet_balances WHERE user_id = $1",
        user_id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected())
}
