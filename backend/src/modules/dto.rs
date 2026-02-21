use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize,Debug)]
pub struct TokenDesign{
  pub google_sub: String,
  pub email: String,
  pub iat: i64,
  pub exp: i64,
}

impl TokenDesign{
  pub fn new(google_sub:String,email:String)->TokenDesign{
    TokenDesign{
      google_sub,
      email,
      iat: Utc::now().timestamp(),
      exp: Utc::now().timestamp()
    }
  }
}