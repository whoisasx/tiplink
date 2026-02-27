use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OAuthCallbackQuery {
  pub code: String,
  pub state: String,
}

#[derive(Deserialize, Debug)]
pub struct GoogleOAuthTokenResponse {
  pub access_token: String,
  pub expires_in: i64,
  pub refresh_token: String,
  pub scope: String,
  pub token_type: String,
  pub id_token: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GoogleUserInfo {
  pub email: String,
  pub name: String,
  pub id: String,
  pub picture: String,
}

pub struct RefreshTokenRecord {
  pub id: String,
  pub user_id: String,
  pub token_hash: String,
  pub expires_at: i64,
}

pub struct UserRecord {
  pub id: String,
  pub google_sub: String,
  pub email: String,
  pub display_name: Option<String>,
  pub avatar_url: Option<String>,
  pub wallet: Option<String>,
}