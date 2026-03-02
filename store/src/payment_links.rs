use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::{dto::PaymentLinkRow, pool::pool};

pub async fn insert_payment_link(
    creator_id: Uuid,
    link_token: &str,
    escrow_pubkey: &str,
    encrypted_escrow_secret: &str,
    mint: Option<&str>,
    amount: i64,
    note: Option<&str>,
    expiry_at: Option<DateTime<Utc>>,
) -> Result<PaymentLinkRow, sqlx::Error> {
    sqlx::query_as!(
        PaymentLinkRow,
        r#"
        INSERT INTO payment_links
          (creator_id, link_token, escrow_pubkey, encrypted_escrow_secret,
          mint, amount, note, expiry_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
        creator_id,
        link_token,
        escrow_pubkey,
        encrypted_escrow_secret,
        mint,
        amount,
        note,
        expiry_at,
    )
    .fetch_one(pool())
    .await
}

pub async fn find_payment_link_by_id(
    id: Uuid,
) -> Result<Option<PaymentLinkRow>, sqlx::Error> {
    sqlx::query_as!(
        PaymentLinkRow,
        "SELECT * FROM payment_links WHERE id = $1",
        id,
    )
    .fetch_optional(pool())
    .await
}

pub async fn find_payment_link_by_token(
    link_token: &str,
) -> Result<Option<PaymentLinkRow>, sqlx::Error> {
    sqlx::query_as!(
        PaymentLinkRow,
        "SELECT * FROM payment_links WHERE link_token = $1",
        link_token,
    )
    .fetch_optional(pool())
    .await
}

pub async fn find_payment_links_by_creator(
    creator_id: Uuid,
) -> Result<Vec<PaymentLinkRow>, sqlx::Error> {
    sqlx::query_as!(
        PaymentLinkRow,
        "SELECT * FROM payment_links WHERE creator_id = $1 ORDER BY created_at DESC",
        creator_id,
    )
    .fetch_all(pool())
    .await
}

pub async fn claim_payment_link(
    link_token: &str,
    claimer_wallet: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE payment_links
        SET status        = 'claimed',
            claimer_wallet = $1,
            claimed_at    = NOW(),
            updated_at    = NOW()
        WHERE link_token = $2
          AND status    = 'active'
          AND (expiry_at IS NULL OR expiry_at > NOW())
        "#,
        claimer_wallet,
        link_token,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn cancel_payment_link(id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE payment_links
        SET status = 'cancelled', updated_at = NOW()
        WHERE id = $1 AND status = 'active'
        "#,
        id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Cancel a link atomically, but only if the caller is the creator.
/// Returns `false` if the link is not active, doesn't exist, or is owned by
/// someone else — all three cases are safe no-ops.
pub async fn cancel_payment_link_owned(id: Uuid, creator_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE payment_links
        SET status = 'cancelled', updated_at = NOW()
        WHERE id = $1 AND creator_id = $2 AND status = 'active'
        "#,
        id,
        creator_id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Revert a previously-claimed link back to 'active'.
/// Used when the MPC transfer fails after the DB was already updated.
pub async fn revert_claim_payment_link(link_token: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE payment_links
        SET status        = 'active',
            claimer_wallet = NULL,
            claimed_at    = NULL,
            updated_at    = NOW()
        WHERE link_token = $1 AND status = 'claimed'
        "#,
        link_token,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Revert a previously-cancelled link back to 'active'.
/// Used when the MPC refund transfer fails after the DB was already updated.
pub async fn revert_cancel_payment_link(id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE payment_links
        SET status = 'active', updated_at = NOW()
        WHERE id = $1 AND status = 'cancelled'
        "#,
        id,
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected() > 0)
}


pub async fn expire_stale_links() -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE payment_links SET status = 'expired', updated_at = NOW() WHERE status = 'active' AND expiry_at IS NOT NULL AND expiry_at <= NOW()"
    )
    .execute(pool())
    .await?;

    Ok(result.rows_affected())
}
