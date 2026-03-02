use actix_web::{
    Error, ResponseError, body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};

use crate::config::Config;
use crate::errors::MpcError;

/// Validates the `X-MPC-Secret` header on every request to the MPC server.
/// Uses constant-time comparison to prevent timing-attack leaks.
pub async fn mpc_auth_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let secret = req
        .app_data::<web::Data<Config>>()
        .map(|c| c.mpc_secret.clone())
        .unwrap_or_default();

    let provided = req
        .headers()
        .get("X-MPC-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Constant-time comparison: reject if secret is empty, lengths differ, or
    // any byte differs.
    let valid = !secret.is_empty()
        && secret.len() == provided.len()
        && secret
            .bytes()
            .zip(provided.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0;

    if !valid {
        let (req, _) = req.into_parts();
        let res = MpcError::Unauthorized.error_response();
        return Ok(ServiceResponse::new(req, res));
    }

    next.call(req).await
}
