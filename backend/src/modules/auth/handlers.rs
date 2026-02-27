use crate::{ modules::{JwtClaims, auth::dto::{GoogleOAuthTokenResponse, GoogleUserInfo, RefreshTokenRecord, UserRecord}}};


pub async fn upsert_user(token_info: &GoogleOAuthTokenResponse, user_info: &GoogleUserInfo) -> bool {
  /*
    - update or insert the user in database.
    - extract the inforamations required from the token_info & user_info
  */
  true
}

pub async fn upsert_wallet(token_info: &GoogleOAuthTokenResponse, user_info: &GoogleUserInfo) -> bool {
  /*
    - update or insert the wallet for the user in database.
  */
  true
}

pub async fn upsert_refresh_token(token_info: &GoogleOAuthTokenResponse, user_info: &GoogleUserInfo) -> bool {
  /*
    - update or insert the refresh token for the user 
  */
  true
}

pub fn create_jwt_token(claims: JwtClaims) -> String {
  "".into()
}

pub fn verify_jwt_token(token: String) -> Result<JwtClaims, String> {
  Err("Invalid token".to_string())
}

pub fn hash_refresh_token(token: &String) -> String {
  String::from("")
}

pub async fn get_refresh_token_record(hashed_token: &String) -> Option<RefreshTokenRecord> {
  /*
    - read the refresh token for the given hash of in database
  */
  None
}

pub async fn get_user_record(user_id: &String) -> Option<UserRecord> {
  /*
    - read the user table for the given user_id from the database.
  */
  None
}