use serde::{Deserialize, Serialize};
use serde_json::Value;
pub type HeliusWebhookPayload = Vec<EnhancedTransaction>;
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedTransaction {
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub txn_type: Option<String>,
    pub source: Option<String>,
    pub fee: Option<u64>,
    pub fee_payer: Option<String>,
    pub signature: String,
    pub slot: Option<u64>,
    pub timestamp: Option<i64>,
    pub native_transfers: Option<Vec<NativeTransfer>>,
    pub token_transfers: Option<Vec<TokenTransfer>>,
    pub account_data: Option<Vec<AccountData>>,
    pub instructions: Option<Vec<Value>>,
    pub events: Option<Value>,
    pub transaction_error: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransfer {
    pub from_user_account: Option<String>,
    pub to_user_account: Option<String>,
    pub from_token_account: Option<String>,
    pub to_token_account: Option<String>,
    pub token_amount: f64,
    pub mint: String,
    pub token_standard: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    pub account: String,
    pub native_balance_change: i64,
    pub token_balance_changes: Option<Vec<TokenBalanceChange>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceChange {
    pub user_account: String,
    pub token_account: String,
    pub mint: String,
    pub raw_token_amount: RawTokenAmount,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTokenAmount {
    pub token_amount: String,
    pub decimals: u8,
}
