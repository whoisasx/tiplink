use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenDesign{
  pub google_sub: String,
  pub email: String,
  pub wallet: String,
  pub iat: i64,
  pub exp: i64,
}

impl TokenDesign{
  pub fn new(google_sub:String,email:String, wallet:String)->TokenDesign{
    TokenDesign{
      google_sub,
      email,
      wallet,
      iat: Utc::now().timestamp(),
      exp: Utc::now().timestamp()
    }
  }
}