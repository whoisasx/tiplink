use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct CreateWalletResponse {
    pub pubkey: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignerInfo {
    User { user_id: String },
    Escrow,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub mint: Option<String>,
    pub signer: SignerInfo,
    pub payer: String,
    pub escrow_hmac: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct SignAndSendRequest {
    pub user_id: String,
    pub wallet_pubkey: String,
    pub transaction_base64: String,
}

#[derive(Debug, Serialize)]
pub struct SignAndSendResponse {
    pub signature: String,
}
