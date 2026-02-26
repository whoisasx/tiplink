use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, middleware::from_fn, post, web::{self, Json, Query}};

use crate::{middlewares::auth_middleware, modules::JwtClaims, utils::{ApiResponse, AppError}};
use super::dto::*;
use super::services::*;

#[get("/quote")]
pub async fn get_swap_quote(req:HttpRequest, query:Query<SwapQuoteQuery>) ->Result<HttpResponse,AppError>{
  match req.extensions().get::<JwtClaims>().cloned() {
    Some(t)=>t,
    None=> return Err(AppError::Unauthorized("Invalid JWT token".to_string()))
  };

  let SwapQuoteQuery {input_mint, output_mint, amount, slippage_bps}=query.into_inner();
  if amount<=0 || !output_mint.len()>0 || !input_mint.len()>0{
    return Err(AppError::BadRequest(String::from("Invalid input/s.")))
  }

  if !validate_mints(&input_mint, &output_mint).await {
    return Err(AppError::BadRequest(String::from("Invalid input/output mints.")))
  }

  let quote_response=get_jupiter_quote(input_mint, output_mint, slippage_bps, amount).await.map_err(|_| AppError::Internal)?;

  Ok(ApiResponse::ok("Quote generated.", quote_response))
}


#[post("/execute")]
pub async fn execute_swap(req:HttpRequest, info: Json<SwapQuoteResponse>) -> Result<HttpResponse,AppError> {
  let token_claims=match req.extensions().get::<JwtClaims>().cloned() {
    Some(t)=>t,
    None=> return Err(AppError::Unauthorized("Invalid JWT token".to_string()))
  };

  let info=info.into_inner();
  let execute_response=execute_quote(info,token_claims).await.map_err(|_| AppError::Internal)?;

  Ok(ApiResponse::ok("Swap execution.", execute_response))
}

#[get("/history")]
pub async fn get_swap_history(req: HttpRequest, query:Query<SwapHistoryQuery>) -> Result<HttpResponse,AppError> {
  let token_claims=match req.extensions().get::<JwtClaims>().cloned() {
    Some(t)=>t,
    None=> return Err(AppError::Unauthorized("Invalid JWT token".to_string()))
  };

  let query=query.into_inner();
  let swap_history=find_swap_history(query,token_claims).await.map_err(|_| AppError::Internal)?;

  Ok(ApiResponse::ok("Swap history fetched.", swap_history))
}

pub fn configure_swap_routes(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("swap")
    .wrap(from_fn(auth_middleware))
    .service(get_swap_quote)
    .service(execute_swap)
    .service(get_swap_history)
  );
}