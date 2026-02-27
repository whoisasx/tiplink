use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::{dto::{RefreshTokenRow, UserRow}, pool::pool};

pub async fn find_user_by_id(id: Uuid) -> Result<Option<UserRow>, sqlx::Error> {
  let user = sqlx::query_as!(
    UserRow,
    "SELECT * FROM users WHERE id = $1",
    id
  )
  .fetch_optional(pool())
  .await?;

  Ok(user)
}

pub async fn find_user_by_google_sub(google_sub: &str) -> Result<Option<UserRow>, sqlx::Error> {
  sqlx::query_as!(
    UserRow,
    "SELECT * FROM users WHERE google_sub = $1",
    google_sub
  )
  .fetch_optional(pool())
  .await
}

pub async fn upsert_user(
  google_sub: &str,
  email: &str,
  display_name: &str,
  avatar_url: &str,
) -> Result<UserRow, sqlx::Error> {
  sqlx::query_as!(
    UserRow,
    r#"
    INSERT INTO users(google_sub, email, display_name, avatar_url)
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (google_sub)
    DO UPDATE
      SET display_name=EXCLUDED.display_name,
          avatar_url=EXCLUDED.avatar_url,
          updated_at=NOW()
    RETURNING *
    "#,
    google_sub,
    email,
    display_name,
    avatar_url
  )
  .fetch_one(pool())
  .await
}

pub async fn update_user_wallet(user_id: Uuid, wallet_pubkey: &str) -> Result<bool, sqlx::Error> {
  let result = sqlx::query!(
    "
    UPDATE users
    SET wallet_pubkey = $1, updated_at = NOW()
    WHERE id = $2
    ",
    wallet_pubkey,
    user_id
  )
  .execute(pool())
  .await?;

  Ok(result.rows_affected() > 0)
}

pub async fn find_refresh_token(token_hash: &str) -> Result<Option<RefreshTokenRow>, sqlx::Error> {
  sqlx::query_as!(
    RefreshTokenRow,
    "SELECT * FROM refresh_tokens WHERE token_hash = $1 AND revoked = FALSE",
    token_hash
  )
  .fetch_optional(pool())
  .await
}

pub async fn insert_refresh_token(
  user_id: Uuid,
  token_hash: &str,
  expires_at: DateTime<Utc>,
  user_agent: Option<&str>,
  ip_address: Option<&str>,
) -> Result<bool, sqlx::Error> {
  let result = sqlx::query!(
    "
    INSERT INTO refresh_tokens (user_id, token_hash, expires_at, user_agent, ip_address)
    VALUES ($1, $2, $3, $4, $5)
    ",
    user_id,
    token_hash,
    expires_at,
    user_agent,
    ip_address
  )
  .execute(pool())
  .await?;

  Ok(result.rows_affected() > 0)
}

pub async fn revoke_refresh_token(token_hash: &str) -> Result<bool, sqlx::Error> {
  let result = sqlx::query!(
    "
    UPDATE refresh_tokens
    SET revoked = TRUE, revoked_at = NOW()
    WHERE token_hash = $1
    ",
    token_hash
  )
  .execute(pool())
  .await?;

  Ok(result.rows_affected() > 0)
}

pub async fn revoke_all_user_tokens(user_id: Uuid) -> Result<bool, sqlx::Error> {
  let result = sqlx::query!(
    "
    UPDATE refresh_tokens
    SET revoked = TRUE, revoked_at = NOW()
    WHERE user_id = $1 AND revoked = FALSE
    ",
    user_id
  )
  .execute(pool())
  .await?;

  Ok(result.rows_affected() > 0)
}