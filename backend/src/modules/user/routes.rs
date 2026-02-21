use actix_web::{HttpRequest, HttpResponse, Responder, get, middleware::from_fn, web, HttpMessage};
use serde_json::json;

use crate::{middlewares::auth_middleware, modules::{TokenDesign, user::services::fetch_user_details}, utils::{Response, send_response}};

#[get("/me")]
pub async fn get_user(req:HttpRequest) -> impl Responder {
  let token_claims =match req.extensions().get::<TokenDesign>().cloned(){
    Some(t)=>t,
    None =>{
      return send_response(Response::new(false,String::from("invalid jwt token."),401,None::<String>))
    }
  };

  let user_record= match fetch_user_details(&token_claims.google_sub).await {
    Some(u) => u,
    None=>{
      return send_response(Response::new(false,String::from("internal server error"),500,None::<String>));
    }
  };

  send_response(Response { 
    success: true, 
    message: String::from("user details fetched"), 
    status_code: 200, 
    value: Some(json!(user_record)) 
  })
}

pub fn scoped_user(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("user")
      .service(get_user)
      .wrap(from_fn(auth_middleware))
  );
}