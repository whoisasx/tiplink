use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;
use crate::{dto::TransactionRow, pool::pool};

pub async fn insert_transaction(
    user_id: Uuid,
    txn_type: &str,
    amount: i64,
    mint: Option<&str>,
    from_address: Option<&str>,
    to_address: Option<&str>,
    fee: Option<i64>,
    metadata: Option<Value>,
) -> Result<TransactionRow, sqlx::Error> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        INSERT INTO transactions
          (user_id, type, amount, mint, from_address, to_address, fee, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
          id, user_id, signature,
          type AS txn_type,
          status, amount, mint, from_address, to_address, fee,
          metadata AS "metadata: Value",
          confirmed_at, created_at, updated_at
        "#,
        user_id,
        txn_type,
        amount,
        mint,
        from_address,
        to_address,
        fee,
        metadata,
    )
    .fetch_one(pool())
    .await
}

pub async fn find_transaction_by_id(
    id: Uuid,
) -> Result<Option<TransactionRow>, sqlx::Error> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        SELECT
          id, user_id, signature,
          type AS txn_type,
          status, amount, mint, from_address, to_address, fee,
          metadata AS "metadata: Value",
          confirmed_at, created_at, updated_at
        FROM transactions
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool())
    .await
}

pub async fn find_transactions_by_user(
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<TransactionRow>, sqlx::Error> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        SELECT
          id, user_id, signature,
          type AS txn_type,
          status, amount, mint, from_address, to_address, fee,
          metadata AS "metadata: Value",
          confirmed_at, created_at, updated_at
        FROM transactions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset,
    )
    .fetch_all(pool())
    .await
}

pub async fn update_transaction_status(
    id: Uuid,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE transactions SET status = $1, updated_at = NOW() WHERE id = $2",
        status,
        id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn confirm_transaction(
    id: Uuid,
    signature: &str,
    confirmed_at: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE transactions
        SET status = 'confirmed',
            signature = $1,
            confirmed_at = $2,
            updated_at = NOW()
        WHERE id = $3
        "#,
        signature,
        confirmed_at,
        id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn find_pending_transactions(
    user_id: Uuid,
) -> Result<Vec<TransactionRow>, sqlx::Error> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        SELECT
          id, user_id, signature,
          type AS txn_type,
          status, amount, mint, from_address, to_address, fee,
          metadata AS "metadata: Value",
          confirmed_at, created_at, updated_at
        FROM transactions
        WHERE user_id = $1 AND status = 'pending'
        ORDER BY created_at ASC
        "#,
        user_id,
    )
    .fetch_all(pool())
    .await
}

pub async fn find_swap_transactions_by_user(
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<TransactionRow>, sqlx::Error> {
    sqlx::query_as!(
        TransactionRow,
        r#"
        SELECT
          id, user_id, signature,
          type AS txn_type,
          status, amount, mint, from_address, to_address, fee,
          metadata AS "metadata: Value",
          confirmed_at, created_at, updated_at
        FROM transactions
        WHERE user_id = $1 AND type = 'swap'
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset,
    )
    .fetch_all(pool())
    .await
}

pub async fn count_swap_transactions_by_user(
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT COUNT(*) AS count FROM transactions WHERE user_id = $1 AND type = 'swap'",
        user_id,
    )
    .fetch_one(pool())
    .await?;

    Ok(row.count.unwrap_or(0))
}

pub async fn count_transactions_by_user(
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT COUNT(*) AS count FROM transactions WHERE user_id = $1",
        user_id,
    )
    .fetch_one(pool())
    .await?;

    Ok(row.count.unwrap_or(0))
}
