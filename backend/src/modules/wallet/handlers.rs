use actix_web::{HttpResponse, web::Query};
use super::dto::*;
use crate::{modules::*, utils::AppError};

pub async fn get_all_wallet_balances(token_claims: JwtClaims) ->Result<HttpResponse,AppError> {
  // WalletBalanceResponse {
  //   wallet_pubkey: wallet.clone(),
  //   total_usd_value: String::from(""),
  //   balances: [].to_vec(),
  //   last_synced_at: Utc::now(),
  // }
  Err(AppError::Internal)
}

pub async fn get_wallet_token_balance(token_claims: JwtClaims, mint: String) -> Result<HttpResponse, AppError>{
  Err(AppError::Internal)
}

pub async fn get_paginated_transactions(token_claims: JwtClaims, query: Query<TransactionFilterQuery>) -> Result<HttpResponse,AppError> {
  // (
  //   Vec::new(),
  //   Pagination {
  //     page: 1,
  //     limit: 1,
  //     total: 1,
  //     total_pages: 1,
  //     has_next: false,
  //     has_prev: true,
  //   }
  // )
  Err(AppError::Internal)
}

pub async fn get_transaction_detail(token_claims: JwtClaims, txn_id: String) -> Result<HttpResponse, AppError> {
  Err(AppError::Internal)
}

pub fn is_valid_solana_address(address: &String) -> bool {
  false
}

pub async fn estimate_transaction_fee(info: EstimateTransactionFeeRequest) -> Result<HttpResponse, AppError> {
  Err(AppError::Internal)
}

pub async fn send_transaction(info: SendTransactionRequest) -> Result<HttpResponse, AppError> {
  Err(AppError::Internal)
}