use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetQuoteQuery{
  pub input_mint: String,
  pub output_min: String,
  pub amount: u64,
  slippage_bps: u16
}

#[derive(Serialize)]
pub struct QuoteQueryResponse{
  pub input_mint: String,
  pub output_mint: String,
  pub input_amount: u64,
  pub output_amount: u64,
  pub price_impact: String,
  pub route_label: String,
  pub slippage_bps: u16
}

#[derive(Deserialize)]
pub struct GetSwapHistoryQuery{
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
pub struct SwapHistoryResponse{
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
