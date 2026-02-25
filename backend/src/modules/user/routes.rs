use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, middleware::from_fn, web};

use crate::{middlewares::auth_middleware, modules::JwtClaims, utils::{ApiResponse, AppError}};
use super::handlers::*;

#[get("/me")]
pub async fn handle_get_current_user(req: HttpRequest) -> Result<HttpResponse, AppError> {
  let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized("Invalid JWT token".to_string()))
    }
  };

  let user_profile = match get_user_profile(&token_claims.id).await {
    Some(u) => u,
    None => {
      return Err(AppError::Internal)
    }
  };

  Ok(ApiResponse::ok("User details fetched.", user_profile))
}

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("user")
      .wrap(from_fn(auth_middleware))
      .service(handle_get_current_user)
  );
}