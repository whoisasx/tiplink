use crate::modules::auth::dto::{GoogleTokenResponse, GoogleUserResponse};


pub async fn save_or_create_user(token_info:GoogleTokenResponse, user_info:GoogleUserResponse)->bool{
  true
}