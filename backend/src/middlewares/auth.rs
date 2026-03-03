use actix_web::{ Error, HttpMessage, ResponseError, body::BoxBody, dev::{ServiceRequest, ServiceResponse}, http::header, middleware::Next, web};

use crate::config::Config;
use crate::modules::auth::*;
use crate::utils::*;

/// Hard-rejects requests that have no valid JWT.
pub async fn auth_middleware(req: ServiceRequest, next: Next<BoxBody>) -> Result<ServiceResponse<BoxBody>, Error> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => Some(h.trim_start_matches("Bearer ").to_string()),
        _ => None,
    };

    let token = match token {
        Some(t) => t,
        None => {
            let (req, _) = req.into_parts();
            let res = AppError::Unauthorized("JWT token not provided.".to_string()).error_response();
            return Ok(ServiceResponse::new(req, res));
        }
    };

    let secret = req
        .app_data::<web::Data<Config>>()
        .map(|c| c.jwt_secret.clone())
        .unwrap_or_default();

    let token_claims = match verify_jwt_token(&token, &secret) {
        Ok(c) => c,
        Err(_) => {
            let (req, _) = req.into_parts();
            let res = AppError::Unauthorized("Invalid JWT token.".to_string()).error_response();
            return Ok(ServiceResponse::new(req, res));
        }
    };

    req.extensions_mut().insert(token_claims);
    next.call(req).await
}

/// Attaches JWT claims to the request extension when a valid Bearer token is
/// present, but **never rejects** the request. Handlers that need auth check
/// extensions themselves and return 401 when claims are absent.
pub async fn attach_claims_middleware(req: ServiceRequest, next: Next<BoxBody>) -> Result<ServiceResponse<BoxBody>, Error> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(str::to_owned);

    if let Some(t) = token {
        let secret = req
            .app_data::<web::Data<Config>>()
            .map(|c| c.jwt_secret.clone())
            .unwrap_or_default();
        if let Ok(claims) = verify_jwt_token(&t, &secret) {
            req.extensions_mut().insert(claims);
        }
    }

    next.call(req).await
}