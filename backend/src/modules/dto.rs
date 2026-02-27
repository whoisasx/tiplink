use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JwtClaims {
  pub id: String,
  pub email: String,
  pub wallet: String,
  pub iat: i64,
  pub exp: i64,
}

impl JwtClaims {
  pub fn new(id: String, email: String, wallet: String) -> JwtClaims {
    JwtClaims {
      id,
      email,
      wallet,
      iat: Utc::now().timestamp(),
      exp: Utc::now().timestamp(),
    }
  }
}