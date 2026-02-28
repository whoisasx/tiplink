use actix_web::HttpResponse;
use uuid::Uuid;

use crate::{modules::JwtClaims, utils::{ApiResponse, AppError}};
use super::dto::UserDetails;

pub async fn get_user_profile(token_claims: JwtClaims) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&token_claims.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID in token".to_string()))?;

    let user = store::users::find_user_by_id(user_id)
        .await
        .map_err(|_| AppError::Internal)?
        .ok_or_else(|| AppError::_NotFound("User not found".to_string()))?;

    let profile = UserDetails {
        email: user.email,
        avatar_url: user.avatar_url.unwrap_or_default(),
        wallet: user.wallet_pubkey.unwrap_or(token_claims.wallet),
        name: user.display_name.unwrap_or_default(),
    };

    Ok(ApiResponse::ok("User profile", profile))
}