use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use store::{
    payment_links::{
        cancel_payment_link_owned, claim_payment_link, find_payment_link_by_token,
        find_payment_links_by_creator, insert_payment_link,
        revert_cancel_payment_link, revert_claim_payment_link,
    },
    transactions::{insert_transaction, update_transaction_signature},
};

use crate::{config::Config, utils::{forward_transfer, AppError, MpcSigner, MpcTransferRequest}};

use super::dto::*;

pub fn is_valid_solana_address(addr: &str) -> bool {
    let len = addr.len();
    len >= 32
        && len <= 44
        && addr
            .chars()
            .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
}

fn generate_link_token() -> String {
    Uuid::new_v4().to_string().replace('-', "")[..16].to_string()
}

fn resolve_symbol(mint: Option<&str>) -> String {
    match mint {
        None | Some("") => String::from("SOL"),
        Some(m) => match m {
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => String::from("USDC"),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => String::from("USDT"),
            other => other.to_string(),
        },
    }
}

fn resolve_status(raw: Option<&str>, expiry_at: Option<chrono::DateTime<Utc>>) -> LinkStatus {
    if raw == Some("active") {
        if let Some(exp) = expiry_at {
            if exp < Utc::now() {
                return LinkStatus::Expired;
            }
        }
        return LinkStatus::Active;
    }
    match raw {
        Some("claimed")   => LinkStatus::Claimed,
        Some("cancelled") => LinkStatus::Cancelled,
        Some("expired")   => LinkStatus::Expired,
        _                 => LinkStatus::Active,
    }
}

pub async fn create_link(
    creator_id: Uuid,
    creator_wallet: &str,
    amount: i64,
    mint: Option<&str>,
    note: Option<&str>,
    expiry_at: Option<chrono::DateTime<Utc>>,
    config: &Config,
) -> Result<CreateLinkResponse, AppError> {
    let escrow = &config.escrow_public_key;

    let link_token = generate_link_token();

    let row = insert_payment_link(
        creator_id,
        &link_token,
        escrow,
        "managed-by-mpc",
        mint,
        amount,
        note,
        expiry_at,
    )
    .await
    .map_err(|e| {
        tracing::error!("insert_payment_link failed: {e}");
        AppError::Internal
    })?;

    let txn_row = insert_transaction(
        creator_id,
        "send",
        amount,
        mint,
        Some(creator_wallet),
        Some(escrow.as_str()),
        None,
        Some(json!({ "link_id": row.id.to_string(), "link_token": link_token, "note": note })),
    )
    .await
    .map_err(|e| {
        tracing::error!("insert_transaction (create_link) failed: {e}");
        AppError::Internal
    })?;

    let signature = forward_transfer(MpcTransferRequest {
        from:   creator_wallet.to_string(),
        to:     escrow.clone(),
        amount,
        mint:   mint.map(String::from),
        signer: MpcSigner::User { user_id: creator_id.to_string() },
        payer:  creator_wallet.to_string(),
    }, config)
    .await?;

    update_transaction_signature(txn_row.id, &signature)
        .await
        .map_err(|e| tracing::error!("update_transaction_signature (create_link) failed: {e}"))
        .ok();

    Ok(CreateLinkResponse {
        link_id:       row.id.to_string(),
        link_token:    row.link_token.clone(),
        link_url:      format!("{}/c/{}", config.base_url, row.link_token),
        escrow_pubkey: escrow.clone(),
        amount:        row.amount as u64,
        mint:          row.mint.clone(),
        note:          row.note.clone(),
        expiry_at:     row.expiry_at.map(|d| d.to_rfc3339()),
        status:        LinkStatus::Active,
        created_at:    row.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
    })
}

pub async fn get_links(
    creator_id: Uuid,
    page: u32,
    limit: u32,
    status_filter: &LinkStatus,
    config: &Config,
) -> Result<MyLinksResponse, AppError> {
    let rows = find_payment_links_by_creator(creator_id)
        .await
        .map_err(|e| {
            tracing::error!("find_payment_links_by_creator failed: {e}");
            AppError::Internal
        })?;

    let filtered: Vec<_> = rows
        .into_iter()
        .filter(|r| {
            let status = resolve_status(r.status.as_deref(), r.expiry_at);
            matches!(
                (status_filter, &status),
                (LinkStatus::All, _)
                    | (LinkStatus::Active,    LinkStatus::Active)
                    | (LinkStatus::Claimed,   LinkStatus::Claimed)
                    | (LinkStatus::Cancelled, LinkStatus::Cancelled)
                    | (LinkStatus::Expired,   LinkStatus::Expired)
            )
        })
        .collect();

    let total       = filtered.len() as u64;
    let total_pages = total.div_ceil(limit as u64);
    let offset      = (page.saturating_sub(1) as usize) * limit as usize;

    let items: Vec<LinkListItem> = filtered
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .map(|r| LinkListItem {
            link_id:        r.id.to_string(),
            link_url:       format!("{}/c/{}", config.base_url, r.link_token),
            amount:         r.amount as u64,
            mint:           r.mint.clone(),
            note:           r.note.clone(),
            status:         resolve_status(r.status.as_deref(), r.expiry_at),
            expiry_at:      r.expiry_at.map(|d| d.to_rfc3339()),
            claimer_wallet: r.claimer_wallet.clone(),
            claimed_at:     r.claimed_at.map(|d| d.to_rfc3339()),
            created_at:     r.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
        })
        .collect();

    Ok(MyLinksResponse {
        pagination: LinkPagination {
            page,
            limit,
            total,
            total_pages,
            has_next: (page as u64) < total_pages,
            has_prev: page > 1,
        },
        links: items,
    })
}

pub async fn lookup_link(link_token: &str, _config: &Config) -> Result<LinkInfoResponse, AppError> {
    let row = find_payment_link_by_token(link_token)
        .await
        .map_err(|e| {
            tracing::error!("find_payment_link_by_token failed: {e}");
            AppError::Internal
        })?
        .ok_or_else(|| AppError::_NotFound(String::from("Link not found")))?;

    let status = resolve_status(row.status.as_deref(), row.expiry_at);
    let symbol = resolve_symbol(row.mint.as_deref());

    Ok(LinkInfoResponse {
        link_id:    row.id.to_string(),
        amount:     row.amount as u64,
        mint:       row.mint,
        symbol,
        note:       row.note,
        status,
        expiry_at:  row.expiry_at.map(|d| d.to_rfc3339()),
        created_at: row.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
    })
}

pub async fn claim_link(
    link_token: &str,
    claimer_wallet: &str,
    config: &Config,
) -> Result<ClaimLinkResponse, AppError> {
    // Read link data (amount, mint, etc.) – still needed for the MPC call
    // even though the atomic UPDATE below is the authoritative gate.
    let row = find_payment_link_by_token(link_token)
        .await
        .map_err(|e| {
            tracing::error!("find_payment_link_by_token failed: {e}");
            AppError::Internal
        })?
        .ok_or_else(|| AppError::_NotFound(String::from("Link not found")))?;

    // Atomically mark the link as claimed BEFORE touching the blockchain.
    // `claim_payment_link` uses WHERE status='active' AND expiry > NOW(), so
    // only one concurrent request can win; all others see rows_affected = 0.
    let claimed = claim_payment_link(link_token, claimer_wallet)
        .await
        .map_err(|e| {
            tracing::error!("claim_payment_link (atomic) failed: {e}");
            AppError::Internal
        })?;

    if !claimed {
        // Surface the most helpful error based on what we read earlier.
        return match resolve_status(row.status.as_deref(), row.expiry_at) {
            LinkStatus::Claimed   => Err(AppError::_Conflict(String::from("Link already claimed"))),
            LinkStatus::Cancelled => Err(AppError::_Conflict(String::from("Link has been cancelled"))),
            LinkStatus::Expired   => Err(AppError::_Conflict(String::from("Link has expired"))),
            _                     => Err(AppError::_Conflict(String::from("Link is no longer available"))),
        };
    }

    let escrow = &config.escrow_public_key;

    let txn_row = insert_transaction(
        row.creator_id,
        "receive",
        row.amount,
        row.mint.as_deref(),
        Some(escrow.as_str()),
        Some(claimer_wallet),
        None,
        Some(json!({ "link_id": row.id.to_string(), "link_token": link_token })),
    )
    .await
    .map_err(|e| {
        tracing::error!("insert_transaction (claim_link) failed: {e}");
        AppError::Internal
    })?;

    // Execute the on-chain transfer. On failure, roll back the DB claim so
    // the link stays claimable rather than being stuck with no on-chain tx.
    let signature = match forward_transfer(MpcTransferRequest {
        from:   escrow.clone(),
        to:     claimer_wallet.to_string(),
        amount: row.amount,
        mint:   row.mint.clone(),
        signer: MpcSigner::Escrow,
        payer:  escrow.clone(),
    }, config).await {
        Ok(sig) => sig,
        Err(e) => {
            if let Err(rv) = revert_claim_payment_link(link_token).await {
                tracing::error!(
                    "CRITICAL: MPC failed AND revert_claim failed for {link_token}: {rv}"
                );
            }
            return Err(e);
        }
    };

    update_transaction_signature(txn_row.id, &signature)
        .await
        .map_err(|e| tracing::error!("update_transaction_signature (claim_link) failed: {e}"))
        .ok();

    Ok(ClaimLinkResponse {
        signature,
        amount:         row.amount as u64,
        mint:           row.mint,
        claimer_wallet: claimer_wallet.to_string(),
        claimed_at:     Utc::now().to_rfc3339(),
    })
}

pub async fn cancel_link(
    link_token: &str,
    user_id: Uuid,
    creator_wallet: &str,
    config: &Config,
) -> Result<CancelLinkResponse, AppError> {
    let row = find_payment_link_by_token(link_token)
        .await
        .map_err(|e| {
            tracing::error!("find_payment_link_by_token failed: {e}");
            AppError::Internal
        })?
        .ok_or_else(|| AppError::_NotFound(String::from("Link not found")))?;

    if row.creator_id != user_id {
        return Err(AppError::_Forbidden);
    }

    // Early status check for a better error message (non-authoritative).
    match resolve_status(row.status.as_deref(), row.expiry_at) {
        LinkStatus::Claimed   => return Err(AppError::_Conflict(String::from("Link already claimed"))),
        LinkStatus::Cancelled => return Err(AppError::_Conflict(String::from("Link already cancelled"))),
        LinkStatus::Expired   => return Err(AppError::_Conflict(String::from("Link has expired"))),
        LinkStatus::Active | LinkStatus::All => {}
    }

    // Atomically cancel (re-checks creator_id and status='active' at DB level)
    // before any on-chain operation.
    let cancelled = cancel_payment_link_owned(row.id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("cancel_payment_link_owned failed: {e}");
            AppError::Internal
        })?;

    if !cancelled {
        return Err(AppError::_Conflict(String::from(
            "Link could not be cancelled — it may have already changed state",
        )));
    }

    let escrow = &config.escrow_public_key;

    let txn_row = insert_transaction(
        user_id,
        "receive",
        row.amount,
        row.mint.as_deref(),
        Some(escrow.as_str()),
        Some(creator_wallet),
        None,
        Some(json!({ "link_id": row.id.to_string(), "link_token": link_token, "action": "cancel" })),
    )
    .await
    .map_err(|e| {
        tracing::error!("insert_transaction (cancel_link) failed: {e}");
        AppError::Internal
    })?;

    let signature = match forward_transfer(MpcTransferRequest {
        from:   escrow.clone(),
        to:     creator_wallet.to_string(),
        amount: row.amount,
        mint:   row.mint.clone(),
        signer: MpcSigner::Escrow,
        payer:  escrow.clone(),
    }, config).await {
        Ok(sig) => sig,
        Err(e) => {
            if let Err(rv) = revert_cancel_payment_link(row.id).await {
                tracing::error!(
                    "CRITICAL: MPC failed AND revert_cancel failed for {link_token}: {rv}"
                );
            }
            return Err(e);
        }
    };

    update_transaction_signature(txn_row.id, &signature)
        .await
        .map_err(|e| tracing::error!("update_transaction_signature (cancel_link) failed: {e}"))
        .ok();

    Ok(CancelLinkResponse {
        signature,
        status: LinkStatus::Cancelled,
    })
}