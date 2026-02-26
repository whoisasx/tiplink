use actix_web::{HttpResponse};

use crate::{modules::{JwtClaims, link::dto::{ClaimLinkRequest, CreateLinkRequest, MyLinksQuery}}, utils::AppError};

pub async fn create_link(info:CreateLinkRequest,token_claims:JwtClaims)->Result<HttpResponse,AppError>{
  Err(AppError::Internal)
}

pub async fn get_all_links(query: MyLinksQuery, token_claims:JwtClaims) -> Result<HttpResponse, AppError>{
  Err(AppError::Internal)
}

pub async fn lookup_link(link_token: String)->Result<HttpResponse,AppError>{
  Err(AppError::Internal)
}

pub async fn claim_link(link_token: String, info: ClaimLinkRequest) ->Result<HttpResponse,AppError>{
  Err(AppError::Internal)
}

pub async fn cancel_link(link_token: String, token_claims:JwtClaims) -> Result<HttpResponse, AppError>{
  Err(AppError::Internal)
}