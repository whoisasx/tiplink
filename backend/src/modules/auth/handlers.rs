use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use uuid::Uuid;

use crate::modules::{JwtClaims, auth::dto::{GoogleOAuthTokenResponse, GoogleUserInfo, RefreshTokenRecord, UserRecord}};

#[derive(Deserialize)]
struct MpcWalletCreateResponse {
    pubkey: String,
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
) -> bool {
    let user = match store::users::find_user_by_google_sub(&user_info.id).await {
        Ok(Some(u)) => u,
        _ => return false,
    };

    // mpc: wallet creation — POST {mpc_url}/wallet/create
    // body: { user_id } → response: { pubkey }
    let resp = match reqwest::Client::new()
        .post(format!("{}/wallet/create", mpc_url))
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

    store::users::update_user_wallet(user.id, &mpc_resp.pubkey)
        .await
        .unwrap_or(false)
}

pub async fn upsert_refresh_token(
    token_info: &GoogleOAuthTokenResponse,
    user_info: &GoogleUserInfo,
) -> bool {
    let user = match store::users::find_user_by_google_sub(&user_info.id).await {
        Ok(Some(u)) => u,
        _ => return false,
    };

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

pub fn create_jwt_token(mut claims: JwtClaims) -> String {
    let secret = env::var("JWT_SECRET").unwrap_or_default();
    let max_age: i64 = env::var("TOKEN_MAXAGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60 * 24 * 3600);

    claims.iat = Utc::now().timestamp();
    claims.exp = Utc::now().timestamp() + max_age;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}

pub fn verify_jwt_token(token: String) -> Result<JwtClaims, String> {
    let secret = env::var("JWT_SECRET").unwrap_or_default();
    decode::<JwtClaims>(
        &token,
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