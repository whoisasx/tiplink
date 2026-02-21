use actix_web::{Error, body::BoxBody, dev::{ServiceRequest, ServiceResponse}, http::header, middleware::Next, HttpMessage};

use crate::{modules::auth::services::verify_jwt_token, utils::{Response, send_response}};

pub async fn auth_middleware(req:ServiceRequest, next: Next<BoxBody>)->Result<ServiceResponse<BoxBody>,Error>{
  let auth_header=req
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|h| h.to_str().ok());

  let token= match auth_header{
    Some(h) if h.starts_with("Bearer ")=>Some(h.trim_start_matches("Bearer ").to_string()),
    _=>None
  };

  let token=match token{
    Some(t)=>t,
    None=>{
      let(req,_)=req.into_parts();
      let res=send_response(Response::new(false,String::from("anauthorised request"),401,None::<String>));
      return Ok(ServiceResponse::new(req,res));
    }
  };

  let token_claims= match verify_jwt_token(token) {
    Ok(c)=>c,
    Err(_msg)=>{
      let(req,_)=req.into_parts();
      let res=send_response(Response::new(false,String::from("anauthorised request"),401,None::<String>));
      return Ok(ServiceResponse::new(req,res));
    }
  };

  req.extensions_mut().insert(token_claims);
  next.call(req).await
}