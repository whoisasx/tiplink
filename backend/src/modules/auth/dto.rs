use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CallbackQuery {
  pub code: String,
  pub state: String
}

#[derive(Deserialize, Debug)]
pub struct GoogleTokenResponse {
  pub access_token: String,
  pub expires_in: i64,
  pub refresh_token: String,
  pub scope: String,
  pub token_type: String,
  pub id_token: Option<String>,
}

#[derive(Deserialize,Debug)]
pub struct GoogleUserResponse{
  pub email: String,
  pub name: String,
  pub id: String,
  pub picture: String
}

pub struct RefreshTokenAuth{
  pub id: String,
  pub user_id: String,
  pub token_hash: String,
  pub expires_at: i64
}

pub struct UserAuth{
  pub id: String,
  pub google_sub: String,
  pub email: String,
  pub wallet:String
}