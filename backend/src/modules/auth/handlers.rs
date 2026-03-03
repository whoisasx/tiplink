use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration as StdDuration;
use uuid::Uuid;

use crate::config::Config;
use crate::modules::{JwtClaims, auth::dto::{GoogleOAuthTokenResponse, GoogleUserInfo, RefreshTokenRecord, UserRecord}};

#[derive(Deserialize)]
struct MpcWalletCreateResponse {
    pubkey: String,
}

// ─── Helius webhook types ─────────────────────────────────────────────────────

/// Shape of the GET /v0/webhooks/{id} response that is relevant to us.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HeliusWebhookResponse {
    webhook_url: String,
    transaction_types: Vec<String>,
    account_addresses: Vec<String>,
    webhook_type: String,
    #[serde(default)]
    auth_header: Option<String>,
}

/// Body sent to PUT /v0/webhooks/{id}.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HeliusWebhookUpdateBody {
    webhook_url: String,
    transaction_types: Vec<String>,
    account_addresses: Vec<String>,
    webhook_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_header: Option<String>,
}

const HELIUS_API_BASE: &str = "https://api-mainnet.helius-rpc.com/v0/webhooks";

/// Appends `new_address` to the Helius webhook identified by `webhook_id`.
///
/// The Helius update-webhook endpoint replaces `accountAddresses` entirely, so
/// we first GET the current webhook to read all existing addresses, then PUT
/// the merged (de-duplicated) list back. Failures are only logged as warnings
/// so they never block the user login flow.
async fn add_address_to_helius_webhook(
    client: &reqwest::Client,
    api_key: &str,
    webhook_id: &str,
    new_address: &str,
) {
    let get_url = format!("{}/{}?api-key={}", HELIUS_API_BASE, webhook_id, api_key);

    let current = match client.get(&get_url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Helius GET webhook failed for address {new_address}: {e}");
            return;
        }
    };

    let webhook: HeliusWebhookResponse = match current.json().await {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("Helius GET webhook JSON parse failed: {e}");
            return;
        }
    };

    // De-duplicate: only append if the address isn't already monitored.
    if webhook.account_addresses.iter().any(|a| a == new_address) {
        tracing::debug!("Address {new_address} already registered on Helius webhook");
        return;
    }

    let mut updated_addresses = webhook.account_addresses;
    updated_addresses.push(new_address.to_string());

    let put_url = format!("{}/{}?api-key={}", HELIUS_API_BASE, webhook_id, api_key);
    let body = HeliusWebhookUpdateBody {
        webhook_url: webhook.webhook_url,
        transaction_types: webhook.transaction_types,
        account_addresses: updated_addresses,
        webhook_type: webhook.webhook_type,
        auth_header: webhook.auth_header,
    };

    match client.put(&put_url).json(&body).send().await {
        Ok(r) if r.status().is_success() => {
            tracing::info!("Registered new wallet {new_address} on Helius webhook {webhook_id}");
        }
        Ok(r) => {
            tracing::warn!(
                "Helius PUT webhook returned {} for address {new_address}",
                r.status()
            );
        }
        Err(e) => {
            tracing::warn!("Helius PUT webhook failed for address {new_address}: {e}");
        }
    }
}

pub async fn upsert_user(
    _token_info: &GoogleOAuthTokenResponse,
    user_info: &GoogleUserInfo,
) -> bool {
    store::users::upsert_user(
        &user_info.id,
        &user_info.email,
        &user_info.name,
        &user_info.picture,
    )
    .await
    .is_ok()
}

pub async fn upsert_wallet(
    _token_info: &GoogleOAuthTokenResponse,
    user_info: &GoogleUserInfo,
    mpc_url: &str,
    mpc_secret: &str,
    helius_api_key: Option<&str>,
    helius_webhook_id: Option<&str>,
) -> bool {
    let user = match store::users::find_user_by_google_sub(&user_info.id).await {
        Ok(Some(u)) => u,
        _ => return false,
    };

    // Remember whether the user already had a wallet before this call.
    // We only register new addresses on the Helius webhook (first-time creation).
    let already_had_wallet = user.wallet_pubkey.is_some();

    let client = match reqwest::Client::builder()
        .timeout(StdDuration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let resp = match client
        .post(format!("{}/wallet/create", mpc_url))
        .header("X-MPC-Secret", mpc_secret)
        .json(&serde_json::json!({ "user_id": user.id.to_string() }))
        .send()
        .await
        .and_then(|r| r.error_for_status())
    {
        Ok(r) => r,
        Err(_) => return false,
    };

    let mpc_resp = match resp.json::<MpcWalletCreateResponse>().await {
        Ok(r) => r,
        Err(_) => return false,
    };

    let saved = store::users::update_user_wallet(user.id, &mpc_resp.pubkey)
        .await
        .unwrap_or(false);

    if !saved {
        return false;
    }

    if !already_had_wallet {
        if let (Some(api_key), Some(webhook_id)) = (helius_api_key, helius_webhook_id) {
            add_address_to_helius_webhook(&client, api_key, webhook_id, &mpc_resp.pubkey).await;
        }
    }

    true
}

pub async fn upsert_refresh_token(
    token_info: &GoogleOAuthTokenResponse,
    user_info: &GoogleUserInfo,
) -> bool {
    let user = match store::users::find_user_by_google_sub(&user_info.id).await {
        Ok(Some(u)) => u,
        _ => return false,
    };

    // Revoke every previous token for this user so we never accumulate stale rows.
    let _ = store::users::revoke_all_user_tokens(user.id).await;

    let token_hash = hash_refresh_token(&token_info.refresh_token);
    let expires_at = Utc::now() + Duration::days(180);

    match store::users::insert_refresh_token(user.id, &token_hash, expires_at, None, None).await {
        Ok(_) => true,
        Err(e) => {
            let msg = e.to_string();
            msg.contains("duplicate") || msg.contains("unique")
        }
    }
}

pub fn create_jwt_token(mut claims: JwtClaims, config: &Config) -> Result<String, String> {
    claims.iat = Utc::now().timestamp();
    claims.exp = Utc::now().timestamp() + config.jwt_max_age;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| e.to_string())
}

pub fn verify_jwt_token(token: &str, secret: &str) -> Result<JwtClaims, String> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| e.to_string())
}

pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn get_refresh_token_record(hashed_token: &str) -> Option<RefreshTokenRecord> {
    let row = store::users::find_refresh_token(hashed_token)
        .await
        .ok()
        .flatten()?;

    Some(RefreshTokenRecord {
        id: row.id.to_string(),
        user_id: row.user_id.to_string(),
        token_hash: row.token_hash,
        expires_at: row.expires_at.timestamp(),
    })
}

pub async fn get_user_record(user_id: &str) -> Option<UserRecord> {
    let uuid = Uuid::parse_str(user_id).ok()?;
    let row = store::users::find_user_by_id(uuid)
        .await
        .ok()
        .flatten()?;

    Some(UserRecord {
        id: row.id.to_string(),
        google_sub: row.google_sub,
        email: row.email,
        display_name: row.display_name,
        avatar_url: row.avatar_url,
        wallet: row.wallet_pubkey,
    })
}