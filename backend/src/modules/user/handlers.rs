use actix_web::HttpResponse;

use crate::{modules::{JwtClaims}, utils::AppError};

pub async fn get_user_profile(token_claims: JwtClaims) -> Result<HttpResponse,AppError>{
  Err(AppError::Internal)
}