use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

use store::transactions::{find_swap_transactions_by_user, count_swap_transactions_by_user, insert_transaction, update_transaction_signature};

use crate::{
    config::Config,
    utils::{
        build_swap_transaction, jupiter_get_quote, forward_swap_sign,
        AppError, MpcSwapSignRequest,
    },
};

use super::dto::*;

pub use crate::utils::is_valid_mint;

pub async fn get_quote(
    input_mint:   &str,
    output_mint:  &str,
    amount:       u64,
    slippage_bps: u16,
    config:       &Config,
) -> Result<SwapQuoteResponse, AppError> {
    let q = jupiter_get_quote(input_mint, output_mint, amount, slippage_bps, config).await?;
    Ok(SwapQuoteResponse {
        input_mint:    q.input_mint,
        output_mint:   q.output_mint,
        input_amount:  q.input_amount,
        output_amount: q.output_amount,
        price_impact:  q.price_impact,
        route_label:   q.route_label,
        slippage_bps:  q.slippage_bps,
        quote_raw:     q.quote_raw,
    })
}

pub async fn execute_swap(
    user_id:     Uuid,
    user_wallet: &str,
    req:         SwapExecuteRequest,
    config:      &Config,
) -> Result<SwapExecuteResponse, AppError> {
    let tx_base64 = build_swap_transaction(&req.quote_raw, user_wallet, config).await?;

    let metadata = json!({
        "input_mint":    req.input_mint,
        "output_mint":   req.output_mint,
        "input_amount":  req.input_amount,
        "output_amount": req.output_amount,
        "slippage_bps":  req.slippage_bps,
        "price_impact":  req.price_impact,
        "route_label":   req.route_label,
    });

    let txn_row = insert_transaction(
        user_id,
        "swap",
        req.input_amount as i64,
        Some(req.input_mint.as_str()),
        Some(user_wallet),
        None,
        None,
        Some(metadata),
    )
    .await
    .map_err(|e| {
        tracing::error!("insert_transaction (execute_swap) failed: {e}");
        AppError::Internal
    })?;

    let signature = forward_swap_sign(
        MpcSwapSignRequest {
            user_id:            user_id.to_string(),
            wallet_pubkey:      user_wallet.to_string(),
            transaction_base64: tx_base64,
        },
        config,
    )
    .await?;

    update_transaction_signature(txn_row.id, &signature)
        .await
        .map_err(|e| tracing::error!("update_transaction_signature (swap) failed: {e}"))
        .ok();

    Ok(SwapExecuteResponse {
        txn_id:    txn_row.id.to_string(),
        signature,
        status:    SwapStatus::Pending,
    })
}

pub async fn get_swap_history(
    user_id: Uuid,
    page:    u32,
    limit:   u32,
) -> Result<SwapHistoryResponse, AppError> {
    let offset = (page.saturating_sub(1) as i64) * limit as i64;

    let rows = find_swap_transactions_by_user(user_id, limit as i64, offset)
        .await
        .map_err(|e| {
            tracing::error!("find_swap_transactions_by_user failed: {e}");
            AppError::Internal
        })?;

    let total = count_swap_transactions_by_user(user_id)
        .await
        .map_err(|e| {
            tracing::error!("count_swap_transactions_by_user failed: {e}");
            AppError::Internal
        })? as u64;

    let total_pages = total.div_ceil(limit as u64);

    let swaps = rows
        .into_iter()
        .map(|r| {
            let meta = r.metadata.as_ref();
            let get_str = |key: &str| -> String {
                meta.and_then(|m| m.get(key))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let output_amount: u64 = meta
                .and_then(|m| m.get("output_amount"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            SwapHistoryItem {
                id:            r.id.to_string(),
                signature:     r.signature,
                status:        r.status.unwrap_or_else(|| "pending".to_string()),
                input_mint:    get_str("input_mint"),
                output_mint:   get_str("output_mint"),
                input_amount:  r.amount as u64,
                output_amount,
                price_impact:  get_str("price_impact"),
                route_label:   get_str("route_label"),
                created_at:    r.created_at.unwrap_or_else(Utc::now).to_rfc3339(),
            }
        })
        .collect();

    Ok(SwapHistoryResponse {
        pagination: SwapPagination {
            page,
            limit,
            total,
            total_pages,
            has_next: (page as u64) < total_pages,
            has_prev: page > 1,
        },
        swaps,
    })
}
