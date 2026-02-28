#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct TokenBalance {
    pub mint:       String,
    pub symbol:     String,
    pub name:       String,
    pub decimal:    String,
    pub raw_amount: i64,
    pub ui_amount:  String,
    pub usd_price:  String,
    pub usd_value:  String,
    pub logo_uri:   String,
}

#[derive(Serialize)]
pub struct WalletBalanceResponse {
    pub wallet_pubkey:   String,
    pub total_usd_value: String,
    pub balances:        Vec<TokenBalance>,
    pub last_synced_at:  DateTime<Utc>,
}

fn default_page()  -> u32 { 1 }
fn default_limit() -> u32 { 20 }

#[derive(Deserialize)]
pub struct TransactionFilterQuery {
    #[serde(default = "default_page")]
    pub page:   u32,
    #[serde(default = "default_limit")]
    pub limit:  u32,
    pub status: Option<String>,
    pub mint:   Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl TransactionStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "confirmed" => TransactionStatus::Confirmed,
            "failed"    => TransactionStatus::Failed,
            _           => TransactionStatus::Pending,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Send,
    Receive,
    Swap,
}

impl TransactionType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "receive" => TransactionType::Receive,
            "swap"    => TransactionType::Swap,
            _         => TransactionType::Send,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    In,
    Out,
}

#[derive(Serialize)]
pub struct TransactionCounterParty {
    pub address:          String,
    pub is_platform_user: bool,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    pub id:                String,
    pub txn_type:          TransactionType,
    pub status:            TransactionStatus,
    pub amount:            i64,
    pub mint:              Option<String>,
    pub symbol:            String,
    pub usd_value_at_time: Option<f64>,
    pub direction:         TransactionDirection,
    pub counter_party:     TransactionCounterParty,
    pub signature:         Option<String>,
    pub block_time:        Option<DateTime<Utc>>,
    pub created_at:        Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct TransactionListResponse {
    pub pagination:   Pagination,
    pub transactions: Vec<TransactionResponse>,
}

#[derive(Serialize)]
pub struct Pagination {
    pub page:        u32,
    pub limit:       u32,
    pub total:       u64,
    pub total_pages: u64,
    pub has_next:    bool,
    pub has_prev:    bool,
}

#[derive(Deserialize)]
pub struct EstimateTransactionFeeRequest {
    pub to_account: String,
    pub amount:     String,
    pub mint:       String,
}

#[derive(Serialize)]
pub struct EstimateFeeResponse {
    pub fee_lamports: u64,
    pub fee_sol:      String,
    pub priority:     String,
}

#[derive(Deserialize)]
pub struct SendTransactionRequest {
    pub to_account: String,
    pub amount:     String,
    pub mint:       String,
}

#[derive(Serialize)]
pub struct SendTransactionResponse {
    pub id:        String,
    pub signature: String,
    pub status:    TransactionStatus,
    pub message:   String,
}