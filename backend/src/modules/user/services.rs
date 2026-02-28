use actix_web::{HttpMessage, HttpRequest, HttpResponse, get};

use crate::{modules::JwtClaims, utils::AppError};
use super::handlers::*;

#[get("/me")]
pub async fn handle_get_current_user(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
        Some(t) => t,
        None => return Err(AppError::Unauthorized("Invalid JWT token".to_string())),
    };

    get_user_profile(token_claims).await
}
