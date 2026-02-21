use chrono::{DateTime, Utc};

pub enum WalletStatus{
  ACTIVE,
  ROTATING,
  REVOKED
}
pub struct UserSchema{
  pub id: String,
  pub email: String,
  pub google_sub: String,
  pub avatar_url: String,
  pub name: String,
  pub pubkey: String
}

pub struct TokenSchema{
  pub id: String,
  pub user_id: String,
  pub token_hash: String,
  pub expires_at: DateTime<Utc>,
  pub revoke: bool,
  pub user_agent: String,
  pub ip_address: String
}

pub struct WalletSchema{
  pub id: String,
  pub user_id: String,
  pub pubkey: String,
  pub shard_index: u64,
  pub encrypted_share: String,
  pub status: WalletStatus
}