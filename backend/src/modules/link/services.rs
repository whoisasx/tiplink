use actix_web::{
    delete, get, post,
    web::{Data, Json, Path, Query},
    HttpMessage, HttpRequest, HttpResponse,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    config::Config,
    modules::JwtClaims,
    utils::{ApiResponse, AppError},
};

use super::dto::*;
use super::handlers::*;

#[post("/create")]
pub async fn handle_create_link(
    req: HttpRequest,
    info: Json<CreateLinkRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
        Some(t) => t,
        None => return Err(AppError::Unauthorized(String::from("Invalid JWT token"))),
    };

    let info = info.into_inner();

    if info.amount == 0 {
        return Err(AppError::BadRequest(String::from("Amount must be greater than zero")));
    }

    let expiry_at = match info.expiry_at.as_deref() {
        Some(s) => {
            let parsed = chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|_| AppError::BadRequest(String::from("Invalid expiry_at — expected ISO-8601")))?
                .with_timezone(&Utc);
            if parsed <= Utc::now() {
                return Err(AppError::BadRequest(String::from("expiry_at must be a future datetime")));
            }
            Some(parsed)
        }
        None => None,
    };

    let creator_id = Uuid::parse_str(&token_claims.id)
        .map_err(|_| AppError::Unauthorized(String::from("Invalid token: bad user ID")))?;

    let response = create_link(
        creator_id,
        &token_claims.wallet,
        info.amount as i64,
        info.mint.as_deref(),
        info.note.as_deref(),
        expiry_at,
        &config,
    )
    .await?;

    Ok(ApiResponse::_created("Payment link created", response))
}

#[get("/my")]
pub async fn handle_get_links(
    req: HttpRequest,
    query: Query<MyLinksQuery>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
        Some(t) => t,
        None => return Err(AppError::Unauthorized(String::from("Invalid JWT token"))),
    };

    let q = query.into_inner();
    let page  = q.page.max(1);
    let limit = q.limit.clamp(1, 100);

    let creator_id = Uuid::parse_str(&token_claims.id)
        .map_err(|_| AppError::Unauthorized(String::from("Invalid token: bad user ID")))?;

    let response = get_links(creator_id, page, limit, &q.status, &config).await?;

    Ok(ApiResponse::ok("Links fetched", response))
}

#[get("/{link_token}")]
pub async fn handle_lookup_link(
    path: Path<String>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let link_token = path.into_inner();

    if link_token.trim().is_empty() {
        return Err(AppError::BadRequest(String::from("Link token is required")));
    }

    let response = lookup_link(&link_token, &config).await?;

    Ok(ApiResponse::ok("Link info", response))
}

#[post("/{link_token}/claim")]
pub async fn handle_claim_link(
    path: Path<String>,
    info: Json<ClaimLinkRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let link_token = path.into_inner();
    let info = info.into_inner();

    if link_token.trim().is_empty() {
        return Err(AppError::BadRequest(String::from("Link token is required")));
    }

    if !is_valid_solana_address(&info.claimer_wallet) {
        return Err(AppError::BadRequest(String::from(
            "Invalid claimer_wallet — must be a valid Solana public key",
        )));
    }

    let response = claim_link(&link_token, &info.claimer_wallet, &config).await?;

    Ok(ApiResponse::ok("Link claimed successfully", response))
}

#[delete("/{link_token}/cancel")]
pub async fn handle_cancel_link(
    req: HttpRequest,
    path: Path<String>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
        Some(t) => t,
        None => return Err(AppError::Unauthorized(String::from("Invalid JWT token"))),
    };

    let link_token = path.into_inner();

    if link_token.trim().is_empty() {
        return Err(AppError::BadRequest(String::from("Link token is required")));
    }

    let user_id = Uuid::parse_str(&token_claims.id)
        .map_err(|_| AppError::Unauthorized(String::from("Invalid token: bad user ID")))?;

    let response = cancel_link(&link_token, user_id, &token_claims.wallet, &config).await?;

    Ok(ApiResponse::ok("Link cancelled", response))
}

