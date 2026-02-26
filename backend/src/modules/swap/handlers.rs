use actix_web::HttpResponse;
use serde_json::json;

use crate::{modules::JwtClaims, utils::AppError};

use super::dto::*;

pub async fn validate_mints(input_mint: &String, output_mint: &String)->bool{
  false
}

pub async fn get_jupiter_quote(input_mint: String, output_mint: String, slippage_bps: u16, amount: u64)->Result<HttpResponse,AppError>{
  Err(AppError::Internal)
}

pub async fn execute_quote(info:SwapQuoteResponse, token_claims: JwtClaims)->Result<HttpResponse,AppError>{
  Err(AppError::Internal)
}

pub async fn find_swap_history(query: SwapHistoryQuery, token_claims: JwtClaims) -> Result<HttpResponse,AppError> {
  Err(AppError::Internal)
}