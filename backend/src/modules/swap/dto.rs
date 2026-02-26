use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Clone)]
pub struct SwapQuoteQuery{
  pub input_mint: String,
  pub output_mint: String,
  pub amount: u64,
  pub slippage_bps: u16,
}

#[derive(Serialize,Deserialize)]
pub struct SwapQuoteResponse{
  pub input_mint: String,
  pub output_mint: String,
  pub input_amount: u64,
  pub output_amount: u64,
  pub price_impact: String,
  pub route_label: String,
  pub slippage_bps: u16,
  pub quote_raw: serde_json::Value,
}

#[derive(Deserialize)]
pub struct SwapHistoryQuery{
  pub page: u32,
  pub limit: u32
}

#[derive(Serialize)]
pub struct SwapPaginated{
  pub page: u32,
  pub limit: u32,
  pub total: u32,
  pub total_pages: u32,
  pub has_next: bool,
  pub has_prev: bool
}
#[derive(Serialize)]
pub enum SwapStatus{
  Pending,
  Confirmed,
  Failed
}

#[derive(Serialize)]
pub struct SwapHistoryList{
  pub id: String,
  pub signature: String,
  pub stattus: SwapStatus,
  pub input_mint: String,
  pub output_mint: String,
  pub input_amount: u64,
  pub output_amount: u64,
  pub price_impact: String,
  pub route_label: String,
  pub created_at: DateTime<Utc>
}

#[derive(Serialize)]
pub struct SwapHistoryResponse{
  pub pagination: SwapPaginated,
  pub swaps: Vec<SwapHistoryList>
}

#[derive(Serialize)]
pub struct SwapExecuteResponse{
  pub txn_id: String,
  pub signature: String,
  pub status: SwapStatus
}
