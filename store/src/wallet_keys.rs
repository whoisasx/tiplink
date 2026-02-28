use uuid::Uuid;
use crate::{dto::WalletKeyRow, pool::pool};

/// Legacy insert — kept for existing callers. Shard data is no longer required
/// by the schema (columns are now nullable) so empty strings / 0 are fine.
pub async fn insert_wallet_key(
    user_id: Uuid,
    pubkey: &str,
    shard_index: i32,
    encrypted_share: &str,
) -> Result<WalletKeyRow, sqlx::Error> {
    sqlx::query_as::<_, WalletKeyRow>(
        r#"
        INSERT INTO wallet_keys (user_id, pubkey, shard_index, encrypted_share)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(pubkey)
    .bind(shard_index)
    .bind(encrypted_share)
    .fetch_one(pool())
    .await
}

/// Insert a wallet key record created by the MPC server.
/// Stores the AES-256-GCM encrypted private key — no shard data needed.
pub async fn insert_wallet_key_with_privkey(
    user_id: Uuid,
    pubkey: &str,
    encrypted_private_key: &str,
) -> Result<WalletKeyRow, sqlx::Error> {
    sqlx::query_as::<_, WalletKeyRow>(
        r#"
        INSERT INTO wallet_keys (user_id, pubkey, encrypted_private_key)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(pubkey)
    .bind(encrypted_private_key)
    .fetch_one(pool())
    .await
}

pub async fn find_wallet_key_by_user_id(
    user_id: Uuid,
) -> Result<Option<WalletKeyRow>, sqlx::Error> {
    sqlx::query_as::<_, WalletKeyRow>(
        "SELECT * FROM wallet_keys WHERE user_id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_optional(pool())
    .await
}

pub async fn find_wallet_key_by_pubkey(
    pubkey: &str,
) -> Result<Option<WalletKeyRow>, sqlx::Error> {
    sqlx::query_as::<_, WalletKeyRow>(
        "SELECT * FROM wallet_keys WHERE pubkey = $1",
    )
    .bind(pubkey)
    .fetch_optional(pool())
    .await
}

pub async fn update_wallet_key_status(
    id: Uuid,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE wallet_keys SET status = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(status)
    .bind(id)
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

