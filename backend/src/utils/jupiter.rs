use reqwest::Client;
use serde::Serialize;
use serde_json::json;

use crate::config::Config;
use super::AppError;

pub fn is_valid_mint(mint: &str) -> bool {
    if mint == "SOL" {
        return true;
    }
    let len = mint.len();
    len >= 32
        && len <= 44
        && mint
            .chars()
            .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
}

fn normalize_mint<'a>(mint: &'a str, sol_mint: &'a str) -> &'a str {
    if mint == "SOL" { sol_mint } else { mint }
}

fn extract_route_label(route_plan: &[serde_json::Value]) -> String {
    route_plan
        .first()
        .and_then(|r| r.get("swapInfo"))
        .and_then(|si| si.get("label"))
        .and_then(|l| l.as_str())
        .unwrap_or("Jupiter")
        .to_string()
}

#[derive(Debug, Serialize)]
pub struct JupiterQuoteResult {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: String,
    pub route_label: String,
    pub slippage_bps: u16,
    pub quote_raw: serde_json::Value,
}

pub async fn get_quote(
    input_mint:   &str,
    output_mint:  &str,
    amount:       u64,
    slippage_bps: u16,
    config:       &Config,
) -> Result<JupiterQuoteResult, AppError> {
    let input  = normalize_mint(input_mint,  &config.sol_mint);
    let output = normalize_mint(output_mint, &config.sol_mint);

    let url = format!(
        "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        config.jupiter_quote_url, input, output, amount, slippage_bps
    );

    let client = Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Jupiter quote request failed: {e}");
            AppError::Internal
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body   = res.text().await.unwrap_or_default();
        tracing::error!("Jupiter quote returned {status}: {body}");
        return Err(AppError::BadRequest(String::from(
            "Jupiter quote unavailable — check mint addresses or try again",
        )));
    }

    let raw: serde_json::Value = res.json().await.map_err(|e| {
        tracing::error!("Jupiter quote parse error: {e}");
        AppError::Internal
    })?;

    let out_amount: u64 = raw
        .get("outAmount")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let price_impact = raw
        .get("priceImpactPct")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    let route_label = raw
        .get("routePlan")
        .and_then(|v| v.as_array())
        .map(|rp| extract_route_label(rp))
        .unwrap_or_else(|| "Jupiter".to_string());

    Ok(JupiterQuoteResult {
        input_mint:    input.to_string(),
        output_mint:   output.to_string(),
        input_amount:  amount,
        output_amount: out_amount,
        price_impact,
        route_label,
        slippage_bps,
        quote_raw:     raw,
    })
}

pub async fn build_swap_transaction(
    quote_raw:   &serde_json::Value,
    user_wallet: &str,
    config:      &Config,
) -> Result<String, AppError> {
    let client = Client::new();

    let body = json!({
        "quoteResponse":    quote_raw,
        "userPublicKey":    user_wallet,
        "wrapAndUnwrapSol": true,
    });

    let res = client
        .post(&config.jupiter_swap_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Jupiter swap request failed: {e}");
            AppError::Internal
        })?;

    if !res.status().is_success() {
        let status = res.status();
        let body   = res.text().await.unwrap_or_default();
        tracing::error!("Jupiter swap returned {status}: {body}");
        return Err(AppError::Internal);
    }

    let data: serde_json::Value = res.json().await.map_err(|e| {
        tracing::error!("Jupiter swap response parse error: {e}");
        AppError::Internal
    })?;

    let tx_base64 = data
        .get("swapTransaction")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            tracing::error!("Jupiter swap response missing swapTransaction field");
            AppError::Internal
        })?
        .to_string();

    Ok(tx_base64)
}
