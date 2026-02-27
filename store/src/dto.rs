use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

pub struct UserRow{
  pub id: Uuid,
  pub google_sub: String,
  pub email: String,
  pub display_name: Option<String>,
  pub avatar_url: Option<String>,
  pub wallet_pubkey: Option<String>,
  pub is_active: Option<bool>,
  pub created_at: Option<DateTime<Utc>>,
  pub updated_at: Option<DateTime<Utc>>
}

pub struct RefreshTokenRow{
  pub id: Uuid,
  pub user_id: Uuid,
  pub token_hash: String,
  pub expires_at: DateTime<Utc>,
  pub revoked: Option<bool>,
  pub revoked_at: Option<DateTime<Utc>>,
  pub user_agent: Option<String>,
  pub ip_address: Option<String>,
  pub created_at: Option<DateTime<Utc>>
}

pub struct WalletKeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub pubkey: String,
    pub shard_index: i32,
    pub encrypted_share: String,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct TransactionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub signature: Option<String>,
    pub txn_type: String,
    pub status: Option<String>,
    pub amount: i64,
    pub mint: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub fee: Option<i64>,
    pub metadata: Option<Value>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub struct WalletBalanceRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub wallet_pubkey: String,
    pub mint: String,
    pub symbol: String,
    pub decimals: i32,
    pub raw_amount: i64,
    pub ui_amount: Option<f64>,
    pub usd_value: Option<f64>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

pub struct SwapQuoteRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub input_mint: String,
    pub output_mint: String,
    pub quoted_amount: i64,
    pub slippage_bps: i32,
    pub price_impact_pct: Option<f64>,
    pub route_plan: Value,
    pub unsigned_tx: String,
    pub expires_at: DateTime<Utc>,
    pub status: Option<String>,
    pub transaction_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
}

pub struct PaymentLinkRow {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub link_token: String,
    pub escrow_pubkey: String,
    pub encrypted_escrow_secret: String,
    pub mint: Option<String>,
    pub amount: i64,
    pub status: Option<String>,
    pub note: Option<String>,
    pub expiry_at: Option<DateTime<Utc>>,
    pub claimer_wallet: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}




