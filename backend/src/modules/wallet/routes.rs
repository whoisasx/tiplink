use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, middleware::from_fn, post, web};
use crate::{
  middlewares::*,
  modules::*,
  utils::*
};
use super::{dto::*, handlers::*};

#[get("/balance")]
pub async fn handle_get_all_balances(req: HttpRequest) -> Result<HttpResponse, AppError> {
  let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
    }
  };

  get_all_wallet_balances(token_claims).await
}

#[get("/balance/{mint}")]
pub async fn handle_get_token_balance(req: HttpRequest, path: web::Path<String>) -> Result<HttpResponse, AppError> {
  let mint = path.into_inner();
  let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
    }
  };

  get_wallet_token_balance(token_claims, mint).await
}

#[get("/transactions")]
pub async fn handle_get_transactions(req: HttpRequest, query: web::Query<TransactionFilterQuery>) -> Result<HttpResponse, AppError> {
  let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
    }
  };

  get_paginated_transactions(token_claims, query).await
}

#[get("/transactions/{id}")]
pub async fn handle_get_transaction(req: HttpRequest, path: web::Path<String>) -> Result<HttpResponse, AppError> {
  let txn_id = path.into_inner();
  let token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
    }
  };
  get_transaction_detail(token_claims, txn_id).await
}

#[post("/send")]
pub async fn handle_send_transaction(req: HttpRequest, info: web::Json<SendTransactionRequest>) -> Result<HttpResponse, AppError> {
  let _token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
    }
  };
  let info=info.into_inner();
  send_transaction(info).await
}

#[post("/send/estimate")]
pub async fn handle_estimate_fee(req: HttpRequest, info: web::Json<EstimateTransactionFeeRequest>) -> Result<HttpResponse, AppError> {
  let _token_claims = match req.extensions().get::<JwtClaims>().cloned() {
    Some(t) => t,
    None => {
      return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
    }
  };

  if !is_valid_solana_address(&info.to_account) {
    return Err(AppError::BadRequest(String::from("Invalid Solana address")));
  }
  let info = info.into_inner();

  estimate_transaction_fee(info).await
}

pub fn configure_wallet_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/wallet")
      .wrap(from_fn(auth_middleware))
      .service(handle_get_all_balances)
      .service(handle_get_token_balance)
      .service(handle_get_transactions)
      .service(handle_get_transaction)
      .service(handle_send_transaction)
      .service(handle_estimate_fee)
  );
}