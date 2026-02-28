use uuid::Uuid;

use store::{
    balances::{find_balance_by_pubkey_and_mint, find_balances_by_user},
    transactions::{
        count_transactions_by_user,
        find_transaction_by_id,
        find_transactions_by_user,
        insert_transaction,
    },
};

use crate::{
    config::Config,
    utils::{forward_transfer, AppError, MpcSigner, MpcTransferRequest},
};

use super::dto::*;

pub fn is_valid_solana_address(address: &str) -> bool {
    let len = address.len();
    len >= 32
        && len <= 44
        && address
            .chars()
            .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
}

pub async fn get_all_wallet_balances(
    user_id:       Uuid,
    wallet_pubkey: &str,
) -> Result<WalletBalanceResponse, AppError> {
    let rows = find_balances_by_user(user_id)
        .await
        .map_err(|e| {
            tracing::error!("find_balances_by_user failed: {e}");
            AppError::Internal
        })?;

    let last_synced_at = rows
        .iter()
        .filter_map(|r| r.last_synced_at)
        .max()
        .unwrap_or_else(chrono::Utc::now);

    let total_usd: f64 = rows.iter().filter_map(|r| r.usd_value).sum();

    let balances = rows
        .into_iter()
        .map(|r| {
            let ui_amount = r.ui_amount.unwrap_or(0.0);
            let usd_value = r.usd_value.unwrap_or(0.0);
            let usd_price = if ui_amount > 0.0 { usd_value / ui_amount } else { 0.0 };
            TokenBalance {
                mint:       r.mint,
                symbol:     r.symbol,
                name:       String::new(),
                decimal:    r.decimals.to_string(),
                raw_amount: r.raw_amount,
                ui_amount:  format!("{:.6}", ui_amount),
                usd_price:  format!("{:.6}", usd_price),
                usd_value:  format!("{:.2}", usd_value),
                logo_uri:   String::new(),
            }
        })
        .collect();

    Ok(WalletBalanceResponse {
        wallet_pubkey:   wallet_pubkey.to_string(),
        total_usd_value: format!("{:.2}", total_usd),
        balances,
        last_synced_at,
    })
}

pub async fn get_wallet_token_balance(
    wallet_pubkey: &str,
    mint:          &str,
) -> Result<TokenBalance, AppError> {
    let row = find_balance_by_pubkey_and_mint(wallet_pubkey, mint)
        .await
        .map_err(|e| {
            tracing::error!("find_balance_by_pubkey_and_mint failed: {e}");
            AppError::Internal
        })?
        .ok_or_else(|| AppError::_NotFound(format!("No balance found for mint {mint}")))?;

    let ui_amount = row.ui_amount.unwrap_or(0.0);
    let usd_value = row.usd_value.unwrap_or(0.0);
    let usd_price = if ui_amount > 0.0 { usd_value / ui_amount } else { 0.0 };

    Ok(TokenBalance {
        mint:       row.mint,
        symbol:     row.symbol,
        name:       String::new(),
        decimal:    row.decimals.to_string(),
        raw_amount: row.raw_amount,
        ui_amount:  format!("{:.6}", ui_amount),
        usd_price:  format!("{:.6}", usd_price),
        usd_value:  format!("{:.2}", usd_value),
        logo_uri:   String::new(),
    })
}

pub async fn get_paginated_transactions(
    user_id: Uuid,
    page:    u32,
    limit:   u32,
) -> Result<TransactionListResponse, AppError> {
    let offset = (page.saturating_sub(1) as i64) * limit as i64;

    let rows = find_transactions_by_user(user_id, limit as i64, offset)
        .await
        .map_err(|e| {
            tracing::error!("find_transactions_by_user failed: {e}");
            AppError::Internal
        })?;

    let total = count_transactions_by_user(user_id)
        .await
        .map_err(|e| {
            tracing::error!("count_transactions_by_user failed: {e}");
            AppError::Internal
        })? as u64;

    let total_pages = total.div_ceil(limit as u64);

    let transactions = rows
        .into_iter()
        .map(|r| {
            let direction = if r.txn_type == "receive" {
                TransactionDirection::In
            } else {
                TransactionDirection::Out
            };
            let counter_party_address = if r.txn_type == "receive" {
                r.from_address.unwrap_or_default()
            } else {
                r.to_address.unwrap_or_default()
            };
            TransactionResponse {
                id:                r.id.to_string(),
                txn_type:          TransactionType::from_str(&r.txn_type),
                status:            TransactionStatus::from_str(r.status.as_deref().unwrap_or("pending")),
                amount:            r.amount,
                mint:              r.mint,
                symbol:            String::new(),
                usd_value_at_time: None,
                direction,
                counter_party: TransactionCounterParty {
                    address:          counter_party_address,
                    is_platform_user: false,
                },
                signature:  r.signature,
                block_time: r.confirmed_at,
                created_at: r.created_at,
            }
        })
        .collect();

    Ok(TransactionListResponse {
        pagination: Pagination {
            page,
            limit,
            total,
            total_pages,
            has_next: (page as u64) < total_pages,
            has_prev: page > 1,
        },
        transactions,
    })
}

pub async fn get_transaction_detail(
    user_id: Uuid,
    txn_id:  Uuid,
) -> Result<TransactionResponse, AppError> {
    let row = find_transaction_by_id(txn_id)
        .await
        .map_err(|e| {
            tracing::error!("find_transaction_by_id failed: {e}");
            AppError::Internal
        })?
        .ok_or_else(|| AppError::_NotFound(String::from("Transaction not found")))?;

    if row.user_id != user_id {
        return Err(AppError::Unauthorized(String::from("Access denied")));
    }

    let direction = if row.txn_type == "receive" {
        TransactionDirection::In
    } else {
        TransactionDirection::Out
    };
    let counter_party_address = if row.txn_type == "receive" {
        row.from_address.unwrap_or_default()
    } else {
        row.to_address.unwrap_or_default()
    };

    Ok(TransactionResponse {
        id:                row.id.to_string(),
        txn_type:          TransactionType::from_str(&row.txn_type),
        status:            TransactionStatus::from_str(row.status.as_deref().unwrap_or("pending")),
        amount:            row.amount,
        mint:              row.mint,
        symbol:            String::new(),
        usd_value_at_time: None,
        direction,
        counter_party: TransactionCounterParty {
            address:          counter_party_address,
            is_platform_user: false,
        },
        signature:  row.signature,
        block_time: row.confirmed_at,
        created_at: row.created_at,
    })
}

pub async fn estimate_transaction_fee(
    info:   &EstimateTransactionFeeRequest,
    config: &Config,
) -> Result<EstimateFeeResponse, AppError> {
    if !is_valid_solana_address(&info.to_account) {
        return Err(AppError::BadRequest(String::from("Invalid destination Solana address")));
    }
    if info.amount.parse::<u64>().map_or(true, |v| v == 0) {
        return Err(AppError::BadRequest(String::from("amount must be a positive integer (lamports)")));
    }

    let fee = config.solana_base_fee;
    Ok(EstimateFeeResponse {
        fee_lamports: fee,
        fee_sol:      format!("{:.9}", fee as f64 / 1e9),
        priority:     String::from("normal"),
    })
}

pub async fn send_transaction(
    user_id:       Uuid,
    wallet_pubkey: &str,
    info:          SendTransactionRequest,
    config:        &Config,
) -> Result<SendTransactionResponse, AppError> {
    if !is_valid_solana_address(&info.to_account) {
        return Err(AppError::BadRequest(String::from("Invalid destination Solana address")));
    }

    let amount: i64 = info
        .amount
        .parse()
        .map_err(|_| AppError::BadRequest(String::from("amount must be a valid integer (lamports)")))?
;
    if amount <= 0 {
        return Err(AppError::BadRequest(String::from("amount must be greater than zero")));
    }

    let mint_opt: Option<&str> = if info.mint.is_empty() || info.mint == "SOL" {
        None
    } else {
        Some(&info.mint)
    };

    let txn_row = insert_transaction(
        user_id,
        "send",
        amount,
        mint_opt,
        Some(wallet_pubkey),
        Some(&info.to_account),
        None,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("insert_transaction (send) failed: {e}");
        AppError::Internal
    })?;

    let signature = forward_transfer(
        MpcTransferRequest {
            from:   wallet_pubkey.to_string(),
            to:     info.to_account.clone(),
            amount,
            mint:   mint_opt.map(|s| s.to_string()),
            signer: MpcSigner::User { user_id: user_id.to_string() },
            payer:  wallet_pubkey.to_string(),
        },
        config,
    )
    .await?;

    Ok(SendTransactionResponse {
        id:        txn_row.id.to_string(),
        signature,
        status:    TransactionStatus::Pending,
        message:   String::from("Transaction submitted"),
    })
}