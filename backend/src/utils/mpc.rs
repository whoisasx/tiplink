use hex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::Config;
use super::AppError;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MpcSigner {
    User { user_id: String },
    Escrow,
}

#[derive(Debug, Serialize)]
pub struct MpcTransferRequest {
    pub from: String,
    pub to: String,
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    pub signer: MpcSigner,
    pub payer: String,
}

#[derive(Debug, Deserialize)]
pub struct MpcTransferResponse {
    pub signature: String,
}

fn compute_escrow_hmac(escrow_pubkey: &str, secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b":");
    hasher.update(escrow_pubkey.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, Serialize)]
pub struct MpcSwapSignRequest {
    pub user_id: String,
    pub wallet_pubkey: String,
    pub transaction_base64: String,
}

#[derive(Debug, Deserialize)]
struct MpcSwapSignResponse {
    pub signature: String,
}

pub async fn forward_swap_sign(req: MpcSwapSignRequest, config: &Config) -> Result<String, AppError> {
    let client = Client::new();

    let res = client
        .post(format!("{}/sign-and-send", config.mpc_server_url))
        .json(&req)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("MPC sign-and-send request failed: {e}");
            AppError::Internal
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        tracing::error!("MPC sign-and-send returned {status}: {body}");
        return Err(AppError::Internal);
    }

    let parsed: MpcSwapSignResponse = res.json().await.map_err(|e| {
        tracing::error!("MPC sign-and-send response parse error: {e}");
        AppError::Internal
    })?;

    Ok(parsed.signature)
}

pub async fn forward_transfer(req: MpcTransferRequest, config: &Config) -> Result<String, AppError> {
    let mpc_url = &config.mpc_server_url;

    let mut body = serde_json::to_value(&req).map_err(|e| {
        tracing::error!("MPC request serialization failed: {e}");
        AppError::Internal
    })?;

    if matches!(req.signer, MpcSigner::Escrow) {
        let hmac = compute_escrow_hmac(&req.from, &config.escrow_hmac_secret);
        if let Some(obj) = body.as_object_mut() {
            obj.insert("escrow_hmac".to_string(), serde_json::Value::String(hmac));
        }
    }

    let client = Client::new();

    let res = client
        .post(format!("{}/transfer", mpc_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("MPC request failed: {e}");
            AppError::Internal
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        tracing::error!("MPC returned {status}: {body}");
        return Err(AppError::Internal);
    }

    let parsed: MpcTransferResponse = res.json().await.map_err(|e| {
        tracing::error!("MPC response parse error: {e}");
        AppError::Internal
    })?;

    Ok(parsed.signature)
}

