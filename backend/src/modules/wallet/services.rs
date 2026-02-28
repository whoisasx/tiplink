use actix_web::{
    get, post,
    web::{Data, Json, Path, Query},
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

#[get("/balance")]
pub async fn handle_get_all_balances(
    req:  HttpRequest,
    _cfg: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let claims = extract_claims(&req)?;
    let user_id = parse_user_id(&claims)?;

    let response = get_all_wallet_balances(user_id, &claims.wallet).await?;
    Ok(ApiResponse::ok("Wallet balances fetched", response))
}

#[get("/balance/{mint}")]
pub async fn handle_get_token_balance(
    req:  HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = extract_claims(&req)?;
    let mint   = path.into_inner();

    if mint.is_empty() {
        return Err(AppError::BadRequest(String::from("mint path parameter is required")));
    }

    let response = get_wallet_token_balance(&claims.wallet, &mint).await?;
    Ok(ApiResponse::ok("Token balance fetched", response))
}

#[get("/transactions")]
pub async fn handle_get_transactions(
    req:   HttpRequest,
    query: Query<TransactionFilterQuery>,
) -> Result<HttpResponse, AppError> {
    let claims  = extract_claims(&req)?;
    let user_id = parse_user_id(&claims)?;

    let q     = query.into_inner();
    let page  = q.page.max(1);
    let limit = q.limit.clamp(1, 100);

    let response = get_paginated_transactions(user_id, page, limit).await?;
    Ok(ApiResponse::ok("Transactions fetched", response))
}

#[get("/transactions/{id}")]
pub async fn handle_get_transaction(
    req:  HttpRequest,
    path: Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims  = extract_claims(&req)?;
    let user_id = parse_user_id(&claims)?;
    let txn_id  = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest(String::from("Invalid transaction ID")))?
;

    let response = get_transaction_detail(user_id, txn_id).await?;
    Ok(ApiResponse::ok("Transaction fetched", response))
}

#[post("/send")]
pub async fn handle_send_transaction(
    req:    HttpRequest,
    info:   Json<SendTransactionRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let claims  = extract_claims(&req)?;
    let user_id = parse_user_id(&claims)?;

    let response = send_transaction(user_id, &claims.wallet, info.into_inner(), &config).await?;
    Ok(ApiResponse::ok("Transaction submitted", response))
}

#[post("/send/estimate")]
pub async fn handle_estimate_fee(
    req:    HttpRequest,
    info:   Json<EstimateTransactionFeeRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, AppError> {
    let _claims = extract_claims(&req)?;

    let response = estimate_transaction_fee(&info, &config).await?;
    Ok(ApiResponse::ok("Fee estimated", response))
}

fn extract_claims(req: &HttpRequest) -> Result<JwtClaims, AppError> {
    req.extensions()
        .get::<JwtClaims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized(String::from("Invalid JWT token")))
}

fn parse_user_id(claims: &JwtClaims) -> Result<Uuid, AppError> {
    Uuid::parse_str(&claims.id)
        .map_err(|_| AppError::Unauthorized(String::from("Invalid token: bad user ID")))
}