use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct WalletRecord {
  pub id: String,
  pub user_id: String,
  pub pubkey: String,
}

#[derive(Serialize, Clone)]
pub struct TokenBalance {
  pub mint: String,
  pub symbol: String,
  pub name: String,
  pub decimal: String,
  pub raw_amount: u128,
  pub ui_amount: String,
  pub usd_price: String,
  pub usd_value: String,
  pub logo_uri: String,
}
#[derive(Serialize)]
pub struct WalletBalanceResponse {
  pub wallet_pubkey: String,
  pub total_usd_value: String,
  pub balances: Vec<TokenBalance>,
  pub last_synced_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub enum TransactionStatus {
  Pending,
  Confirmed,
  Failed,
  All,
}

#[derive(Deserialize, Serialize)]
pub enum TransactionType {
  Send,
  Receive,
  Swap,
}

#[derive(Deserialize, Serialize)]
pub enum TransactionDirection {
  In,
  Out,
} 
#[derive(Deserialize)]
pub struct TransactionFilterQuery {
  pub page: u32,
  pub limit: u32,
  pub status: TransactionStatus,
  pub mint: String,
  pub from: String,
  pub to: String,
}

#[derive(Deserialize, Serialize)]
pub struct TransactionCounterParty {
  pub address: String,
  pub is_platform_user: bool,
}

#[derive(Serialize)]
pub struct TransactionResponse{
  pub id: String,
  pub txn_type: TransactionType,
  pub status: TransactionStatus,
  pub amount: u64,
  pub mint: String,
  pub symbol: String,
  pub usd_value_at_time: u64,
  pub direction: TransactionDirection,
  pub counter_party: TransactionCounterParty,
  pub signature: String,
  pub block_time: DateTime<Utc>,
  pub created_at: DateTime<Utc>
}

#[derive(Serialize)]
pub struct Pagination{
  pub page:u32,
  pub limit: u32,
  pub total: u32,
  pub total_pages: u32,
  pub has_next: bool,
  pub has_prev: bool
}

#[derive(Deserialize)]
pub struct EstimateTransactionFeeRequest {
  pub to_account: String,
  pub amount: String,
  pub mint: String,
}

#[derive(Deserialize)]
pub struct SendTransactionRequest {
  pub to_account: String,
  pub from_account: String,
  pub amount: String,
  pub mint: String,
}

#[derive(Serialize)]
pub struct SendTransactionResponse {
  pub id: String,
  pub signature: String,
  pub status: TransactionStatus,
  pub message: String,
}