#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct SwapQuoteQuery {
    pub input_mint:   String,
    pub output_mint:  String,
    pub amount:       u64,
    pub slippage_bps: u16,
}

fn default_page()  -> u32 { 1 }
fn default_limit() -> u32 { 20 }

#[derive(Deserialize)]
pub struct SwapHistoryQuery {
    #[serde(default = "default_page")]
    pub page:  u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SwapExecuteRequest {
    pub input_mint:    String,
    pub output_mint:   String,
    pub input_amount:  u64,
    pub output_amount: u64,
    pub slippage_bps:  u16,
    pub price_impact:  String,
    pub route_label:   String,
    // Full Jupiter quoteResponse; forwarded back to Jupiter /v6/swap.
    pub quote_raw:     serde_json::Value,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct SwapQuoteResponse {
    pub input_mint:    String,
    pub output_mint:   String,
    pub input_amount:  u64,
    pub output_amount: u64,
    pub price_impact:  String,
    pub route_label:   String,
    pub slippage_bps:  u16,
    /// Opaque Jupiter quoteResponse; pass back as-is in execute request.
    pub quote_raw:     serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SwapStatus {
    Pending,
    Confirmed,
    Failed,
}

impl SwapStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "confirmed" => SwapStatus::Confirmed,
            "failed"    => SwapStatus::Failed,
            _           => SwapStatus::Pending,
        }
    }
}

#[derive(Serialize)]
pub struct SwapPagination {
    pub page:        u32,
    pub limit:       u32,
    pub total:       u64,
    pub total_pages: u64,
    pub has_next:    bool,
    pub has_prev:    bool,
}

#[derive(Serialize)]
pub struct SwapHistoryItem {
    pub id:            String,
    pub signature:     Option<String>,
    pub status:        String,
    pub input_mint:    String,
    pub output_mint:   String,
    pub input_amount:  u64,
    pub output_amount: u64,
    pub price_impact:  String,
    pub route_label:   String,
    pub created_at:    String,
}

#[derive(Serialize)]
pub struct SwapHistoryResponse {
    pub pagination: SwapPagination,
    pub swaps:      Vec<SwapHistoryItem>,
}

#[derive(Serialize)]
pub struct SwapExecuteResponse {
    pub txn_id:    String,
    pub signature: String,
    pub status:    SwapStatus,
}
