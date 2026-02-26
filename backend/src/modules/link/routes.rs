use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, middleware::from_fn, post, web::{self, Json, Path, Query}};
use crate::{middlewares::auth_middleware, modules::JwtClaims, utils::AppError};
use super::dto::*;
use super::services::*;

#[post("/create")]
pub async fn handle_create_link(req:HttpRequest, info:Json<CreateLinkRequest>) -> Result<HttpResponse,AppError>{
  let token_claims=match req.extensions().get::<JwtClaims>().cloned(){
    Some(t)=>t,
    None=> return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
  };
  let info=info.into_inner(); 

  if !info.amount>0 {
    return Err(AppError::BadRequest(String::from("Invalid input/s.")))
  }

  create_link(info,token_claims).await
}

#[get("/my")]
pub async fn handle_get_links(req:HttpRequest, query:Query<MyLinksQuery>)->Result<HttpResponse,AppError>{
  let token_claims=match req.extensions().get::<JwtClaims>().cloned(){
    Some(t)=>t,
    None=> return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
  };
  let query=query.into_inner();

  get_all_links(query, token_claims).await
}

#[get("/{link_token}")]
pub async fn handle_lookup_link(path: Path<String>)->Result<HttpResponse,AppError>{
  let link_token = path.into_inner();

  lookup_link(link_token).await
}

#[post("/{link_token}/claim")]
pub async fn handle_claim_link(path: Path<String>, info:Json<ClaimLinkRequest>)->Result<HttpResponse,AppError>{
  let link_token=path.into_inner();
  let info=info.into_inner();

  claim_link(link_token, info).await
}

#[delete("/{link_token}/cancel")]
pub async fn handle_delete_claim(req:HttpRequest, path:Path<String>) -> Result<HttpResponse,AppError>{
  let token_claims=match req.extensions().get::<JwtClaims>().cloned(){
    Some(t)=>t,
    None=> return Err(AppError::Unauthorized(String::from("Invalid JWT token")))
  };
  let link_token=path.into_inner();

  cancel_link(link_token, token_claims).await
}

pub fn configure_links_routes(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("/link")
      .service(
        web::scope("")
          .wrap(from_fn(auth_middleware))
          .service(handle_create_link)   
          .service(handle_get_links)     
          .service(handle_delete_claim)  
      )
      .service(handle_lookup_link)      
      .service(handle_claim_link)        
  );
}