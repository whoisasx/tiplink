use actix_web::{Error, HttpRequest, HttpResponse, post, web::{Data, Json}};
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use uuid::Uuid;

use super::dto::*;

const SOL_MINT:     &str = "SOL";
const SOL_SYMBOL:   &str = "SOL";
const SOL_DECIMALS: i32  = 9;
const NATIVE_SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[post("/webhook")]
pub async fn handle_hooks(
    _req: HttpRequest,
    rpc_url: Data<String>,
    payload: Json<HeliusWebhookPayload>,
) -> Result<HttpResponse, Error> {
    let txns = payload.into_inner();

    for txn in &txns {
        tracing::info!(
            signature = %txn.signature,
            txn_type  = ?txn.txn_type,
            source    = ?txn.source,
            slot      = ?txn.slot,
            timestamp = ?txn.timestamp,
            "received helius webhook"
        );

        if txn.transaction_error.is_some() {
            tracing::warn!(signature = %txn.signature, "skipping failed transaction");
            continue;
        }

        let confirmed_at: DateTime<Utc> = txn
            .timestamp
            .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
            .unwrap_or_else(Utc::now);

        process_native_transfers(txn, confirmed_at).await;
        process_token_transfers(txn, confirmed_at).await;
        process_balance_changes(txn, &rpc_url).await;
    }

    Ok(HttpResponse::Ok().finish())
}

async fn process_native_transfers(txn: &EnhancedTransaction, confirmed_at: DateTime<Utc>) {
    let transfers = match &txn.native_transfers {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    for transfer in transfers {
        if let Ok(Some(w)) =
            store::wallet_keys::find_wallet_key_by_pubkey(&transfer.to_user_account).await
        {
            upsert_confirmed_txn(
                w.user_id,
                "receive",
                transfer.amount,
                None,
                Some(&transfer.from_user_account),
                Some(&transfer.to_user_account),
                txn.fee.map(|f| f as i64),
                &txn.signature,
                confirmed_at,
            )
            .await;
        }

        if let Ok(Some(w)) =
            store::wallet_keys::find_wallet_key_by_pubkey(&transfer.from_user_account).await
        {
            upsert_confirmed_txn(
                w.user_id,
                "send",
                transfer.amount,
                None,
                Some(&transfer.from_user_account),
                Some(&transfer.to_user_account),
                txn.fee.map(|f| f as i64),
                &txn.signature,
                confirmed_at,
            )
            .await;
        }
    }
}

async fn process_token_transfers(txn: &EnhancedTransaction, confirmed_at: DateTime<Utc>) {
    let transfers = match &txn.token_transfers {
        Some(t) if !t.is_empty() => t,
        _ => return,
    };

    for transfer in transfers {
        let from_acct = transfer.from_user_account.as_deref().unwrap_or("");
        let to_acct = transfer.to_user_account.as_deref().unwrap_or("");

        let ui_amount = transfer.token_amount as i64;

        if !to_acct.is_empty() {
            if let Ok(Some(w)) = store::wallet_keys::find_wallet_key_by_pubkey(to_acct).await {
                upsert_confirmed_txn(
                    w.user_id,
                    "receive",
                    ui_amount,
                    Some(&transfer.mint),
                    if from_acct.is_empty() { None } else { Some(from_acct) },
                    Some(to_acct),
                    txn.fee.map(|f| f as i64),
                    &txn.signature,
                    confirmed_at,
                )
                .await;
            }
        }

        if !from_acct.is_empty() {
            if let Ok(Some(w)) = store::wallet_keys::find_wallet_key_by_pubkey(from_acct).await {
                upsert_confirmed_txn(
                    w.user_id,
                    "send",
                    ui_amount,
                    Some(&transfer.mint),
                    Some(from_acct),
                    if to_acct.is_empty() { None } else { Some(to_acct) },
                    txn.fee.map(|f| f as i64),
                    &txn.signature,
                    confirmed_at,
                )
                .await;
            }
        }
    }
}


async fn process_balance_changes(txn: &EnhancedTransaction, rpc_url: &str) {
    let account_data = match &txn.account_data {
        Some(d) if !d.is_empty() => d,
        _ => return,
    };

    for data in account_data {
        if data.native_balance_change != 0 {
            if let Ok(Some(w)) =
                store::wallet_keys::find_wallet_key_by_pubkey(&data.account).await
            {
                match fetch_sol_balance(rpc_url, &data.account).await {
                    Ok(lamports) => {
                        let ui = lamports as f64 / 1_000_000_000.0;
                        let usd_price = fetch_usd_price(SOL_MINT).await;
                        let usd_value = usd_price.map(|p| ui * p);
                        if let Err(e) = store::balances::upsert_balance(
                            w.user_id,
                            &data.account,
                            SOL_MINT,
                            SOL_SYMBOL,
                            SOL_DECIMALS,
                            lamports,
                            Some(ui),
                            usd_value,
                        )
                        .await
                        {
                            tracing::error!(
                                err     = %e,
                                account = %data.account,
                                "upsert SOL balance failed"
                            );
                        } else {
                            tracing::debug!(
                                account  = %data.account,
                                lamports = lamports,
                                "SOL balance upserted"
                            );
                        }
                    }
                    Err(e) => tracing::error!(
                        err     = %e,
                        account = %data.account,
                        "fetch_sol_balance failed"
                    ),
                }
            }
        }

        let token_changes = match &data.token_balance_changes {
            Some(c) if !c.is_empty() => c,
            _ => continue,
        };

        for change in token_changes {
            let Ok(Some(w)) =
                store::wallet_keys::find_wallet_key_by_pubkey(&change.user_account).await
            else {
                continue;
            };

            let decimals = change.raw_token_amount.decimals as i32;

            let raw_amount = match change.raw_token_amount.token_amount.parse::<i64>() {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(
                        err   = %e,
                        token = %change.raw_token_amount.token_amount,
                        "could not parse raw_token_amount — skipping balance upsert"
                    );
                    continue;
                }
            };

            if raw_amount < 0 {
                continue;
            }

            let ui = raw_amount as f64 / 10_f64.powi(decimals);
            let symbol_len = change.mint.len().min(8);
            let symbol = &change.mint[..symbol_len];

            let usd_price = fetch_usd_price(&change.mint).await;
            let usd_value = usd_price.map(|p| ui * p);

            if let Err(e) = store::balances::upsert_balance(
                w.user_id,
                &change.user_account,
                &change.mint,
                symbol,
                decimals,
                raw_amount,
                Some(ui),
                usd_value,
            )
            .await
            {
                tracing::error!(
                    err  = %e,
                    mint = %change.mint,
                    "upsert_balance failed"
                );
            } else {
                tracing::debug!(
                    mint       = %change.mint,
                    raw_amount = raw_amount,
                    user_id    = %w.user_id,
                    "balance upserted"
                );
            }
        }
    }
}

#[derive(Deserialize)]
struct RpcResponse {
    result: RpcResult,
}

#[derive(Deserialize)]
struct RpcResult {
    value: i64,
}

/// Fetch the current USD price for a mint from Jupiter Price API v2.
/// Pass "SOL" to get the native SOL price.
async fn fetch_usd_price(mint: &str) -> Option<f64> {
    let api_mint = if mint == SOL_MINT {
        NATIVE_SOL_MINT.to_string()
    } else {
        mint.to_string()
    };
    let url = format!("https://lite-api.jup.ag/price/v2?ids={}", api_mint);
    let data: serde_json::Value = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    data.get("data")
        .and_then(|d| d.get(&api_mint))
        .and_then(|info| info.get("price"))
        .and_then(|p| p.as_str())
        .and_then(|s| s.parse::<f64>().ok())
}

async fn fetch_sol_balance(rpc_url: &str, pubkey: &str) -> Result<i64, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id":      1,
        "method":  "getBalance",
        "params":  [pubkey]
    });

    let resp = reqwest::Client::new()
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<RpcResponse>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(resp.result.value)
}

async fn upsert_confirmed_txn(
    user_id: Uuid,
    txn_type: &str,
    amount: i64,
    mint: Option<&str>,
    from_address: Option<&str>,
    to_address: Option<&str>,
    fee: Option<i64>,
    signature: &str,
    confirmed_at: DateTime<Utc>,
) {
    match store::transactions::find_transaction_by_signature(signature).await {
        Ok(Some(row)) => {
            if row.status.as_deref() != Some("confirmed") {
                match store::transactions::confirm_transaction(row.id, signature, confirmed_at)
                    .await
                {
                    Ok(_) => tracing::info!(%signature, "confirmed existing pending transaction"),
                    Err(e) => tracing::error!(err = %e, %signature, "confirm_transaction failed"),
                }
            } else {
                tracing::debug!(%signature, "transaction already confirmed — skipping");
            }
        }

        Ok(None) => {
            match store::transactions::insert_transaction(
                user_id,
                txn_type,
                amount,
                mint,
                from_address,
                to_address,
                fee,
                None,
            )
            .await
            {
                Ok(row) => {
                    match store::transactions::confirm_transaction(row.id, signature, confirmed_at)
                        .await
                    {
                        Ok(_) => tracing::info!(
                            %signature,
                            txn_type,
                            user_id = %user_id,
                            "inserted and confirmed new transaction"
                        ),
                        Err(e) => tracing::error!(
                            err = %e,
                            %signature,
                            "confirm newly inserted transaction failed"
                        ),
                    }
                }
                Err(e) => {
                    tracing::error!(err = %e, %signature, "insert_transaction failed");
                }
            }
        }

        Err(e) => {
            tracing::error!(err = %e, %signature, "find_transaction_by_signature failed");
        }
    }
}
