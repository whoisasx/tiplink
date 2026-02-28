use actix_web::{
    get, post,
    web::{Data, Json, Query},
    HttpMessage, HttpRequest, HttpResponse,
};
use uuid::Uuid;

use crate::{
    config::Config,
    modules::JwtClaims,
    utils::{ApiResponse, AppError},
};

use super::dto::*;
use super::handlers::*;

#[get("/quote")]
pub async fn handle_get_swap_quote(
    req:    HttpRequest,
    query:  Query<SwapQuoteQuery>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    match req.extensions().get::<JwtClaims>().cloned() {
        Some(_) => {}
        None    => return Err(AppError::Unauthorized(String::from("Invalid JWT token"))),
    };

    let q = query.into_inner();

    if q.amount == 0 {
        return Err(AppError::BadRequest(String::from("amount must be greater than zero")));
    }
    if q.input_mint.is_empty() || q.output_mint.is_empty() {
        return Err(AppError::BadRequest(String::from("input_mint and output_mint are required")));
    }
    if q.input_mint == q.output_mint {
        return Err(AppError::BadRequest(String::from("input_mint and output_mint must differ")));
    }
    if !is_valid_mint(&q.input_mint) || !is_valid_mint(&q.output_mint) {
        return Err(AppError::BadRequest(String::from("Invalid input or output mint address")));
    }

    let response = get_quote(&q.input_mint, &q.output_mint, q.amount, q.slippage_bps, &config).await?;

    Ok(ApiResponse::ok("Swap quote fetched", response))
}


#[post("/execute")]
pub async fn handle_execute_swap(
    req:    HttpRequest,
    info:   Json<SwapExecuteRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
        Some(t) => t,
        None    => return Err(AppError::Unauthorized(String::from("Invalid JWT token"))),
    };

    let body = info.into_inner();

    if body.input_amount == 0 {
        return Err(AppError::BadRequest(String::from("input_amount must be greater than zero")));
    }
    if body.input_mint.is_empty() || body.output_mint.is_empty() {
        return Err(AppError::BadRequest(String::from("input_mint and output_mint are required")));
    }
    if body.input_mint == body.output_mint {
        return Err(AppError::BadRequest(String::from("input_mint and output_mint must differ")));
    }

    let user_id = Uuid::parse_str(&token_claims.id)
        .map_err(|_| AppError::Unauthorized(String::from("Invalid token: bad user ID")))?;

    let response = execute_swap(user_id, &token_claims.wallet, body, &config).await?;

    Ok(ApiResponse::ok("Swap executed", response))
}


#[get("/history")]
pub async fn handle_get_swap_history(
    req:   HttpRequest,
    query: Query<SwapHistoryQuery>,
) -> Result<HttpResponse, AppError> {
    let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
        Some(t) => t,
        None    => return Err(AppError::Unauthorized(String::from("Invalid JWT token"))),
    };

    let q     = query.into_inner();
    let page  = q.page.max(1);
    let limit = q.limit.clamp(1, 100);

    let user_id = Uuid::parse_str(&token_claims.id)
        .map_err(|_| AppError::Unauthorized(String::from("Invalid token: bad user ID")))?;

    let response = get_swap_history(user_id, page, limit).await?;

    Ok(ApiResponse::ok("Swap history fetched", response))
}
