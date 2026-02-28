use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use store::transactions::{find_swap_transactions_by_user, count_swap_transactions_by_user, insert_transaction};

use crate::{config::Config, utils::{forward_swap_sign, AppError, MpcSwapSignRequest}};

use super::dto::*;

pub fn is_valid_mint(mint: &str) -> bool {
    if mint == "SOL" { return true; }
    let len = mint.len();
    len >= 32
        && len <= 44
        && mint
            .chars()
            .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
}

fn normalize_mint<'a>(mint: &'a str, sol_mint: &'a str) -> &'a str {
    if mint == "SOL" { sol_mint } else { mint }
}

fn extract_route_label(route_plan: &[serde_json::Value]) -> String {
    route_plan
        .first()
        .and_then(|r| r.get("swapInfo"))
        .and_then(|si| si.get("label"))
        .and_then(|l| l.as_str())
        .unwrap_or("Jupiter")
        .to_string()
}

pub async fn get_quote(
    input_mint:   &str,
    output_mint:  &str,
    amount:       u64,
    slippage_bps: u16,
    config:       &Config,
) -> Result<SwapQuoteResponse, AppError> {
    let input  = normalize_mint(input_mint,  &config.sol_mint);
    let output = normalize_mint(output_mint, &config.sol_mint);

    let url = format!(
        "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        config.jupiter_quote_url, input, output, amount, slippage_bps
    );

    let client = Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Jupiter quote request failed: {e}");
            AppError::Internal
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body   = res.text().await.unwrap_or_default();
        tracing::error!("Jupiter quote returned {status}: {body}");
        return Err(AppError::BadRequest(String::from(
            "Jupiter quote unavailable — check mint addresses or try again",
        )));
    }

    let raw: serde_json::Value = res.json().await.map_err(|e| {
        tracing::error!("Jupiter quote parse error: {e}");
        AppError::Internal
    })?;

    let out_amount: u64 = raw
        .get("outAmount")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let price_impact = raw
        .get("priceImpactPct")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    let route_label = raw
        .get("routePlan")
        .and_then(|v| v.as_array())
        .map(|rp| extract_route_label(rp))
        .unwrap_or_else(|| "Jupiter".to_string());

    Ok(SwapQuoteResponse {
        input_mint:    input.to_string(),
        output_mint:   output.to_string(),
        input_amount:  amount,
        output_amount: out_amount,
        price_impact,
        route_label,
        slippage_bps,
        quote_raw: raw,
    })
}

pub async fn execute_swap(
    user_id:     Uuid,
    user_wallet: &str,
    req:         SwapExecuteRequest,
    config:      &Config,
) -> Result<SwapExecuteResponse, AppError> {
    let client    = Client::new();
    let swap_body = json!({
        "quoteResponse":    req.quote_raw,
        "userPublicKey":    user_wallet,
        "wrapAndUnwrapSol": true,
    });

    let swap_res = client
        .post(&config.jupiter_swap_url)
        .json(&swap_body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Jupiter swap request failed: {e}");
            AppError::Internal
        })?;

    if !swap_res.status().is_success() {
        let status = swap_res.status();
        let body   = swap_res.text().await.unwrap_or_default();
        tracing::error!("Jupiter swap returned {status}: {body}");
        return Err(AppError::Internal);
    }

    let swap_data: serde_json::Value = swap_res.json().await.map_err(|e| {
        tracing::error!("Jupiter swap response parse error: {e}");
        AppError::Internal
    })?;

    let tx_base64 = swap_data
        .get("swapTransaction")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            tracing::error!("Jupiter swap response missing swapTransaction field");
            AppError::Internal
        })?
        .to_string();

    // 2. MPC: sign and broadcast the transaction.
    let signature = forward_swap_sign(
        MpcSwapSignRequest {
            user_id:             user_id.to_string(),
            wallet_pubkey:       user_wallet.to_string(),
            transaction_base64:  tx_base64,
        },
        config,
    )
    .await?;

    let metadata = json!({
        "input_mint":    req.input_mint,
        "output_mint":   req.output_mint,
        "input_amount":  req.input_amount,
        "output_amount": req.output_amount,
        "slippage_bps":  req.slippage_bps,
        "price_impact":  req.price_impact,
        "route_label":   req.route_label,
        "signature":     signature,
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
