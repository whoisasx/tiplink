use crate::{db::dto::UserSchema, modules::{TokenDesign, auth::dto::{GoogleTokenResponse, GoogleUserResponse, RefreshTokenAuth, UserAuth}}};


pub async fn upsert_user(token_info:&GoogleTokenResponse, user_info:&GoogleUserResponse)->bool{
  true
}

pub async fn upsert_wallet(token_info:&GoogleTokenResponse,user_info:&GoogleUserResponse)->String{
  String::from("dummy wallet address")
}

pub async fn upsert_refresh_token(token_info:&GoogleTokenResponse, user_info:&GoogleUserResponse)->bool{
  true
}

pub fn create_jwt_token(token_design: TokenDesign)->String{
  "".into()
}

pub fn verify_jwt_token(token:String)->Result<TokenDesign,String>{
  Err("Invalid token".to_string())
}

pub fn hash_token(token:&String)->String{
  String::from("")
}

pub async fn get_refresh_token_details(hashed_token:&String) -> Option<RefreshTokenAuth> {
    None
}

pub async fn get_user_details(user_id: &String)->Option<UserAuth>{
  None
}