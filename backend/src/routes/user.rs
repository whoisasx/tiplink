use actix_web::{HttpRequest, HttpResponse, Responder, delete, get, post, web};
use crate::utils::*;

#[post("/sign-up")]
pub async fn sign_up(_req: HttpRequest)->impl Responder{
  let response: Response<()> = Response{
    success: true,
    status_code: 200,
    message: "ok".into(),
    data: Option::None
  };
  send_response(response)
}
#[post("/sign-in")]
pub async fn sign_in(_req: HttpRequest)->impl Responder{
  HttpResponse::Ok().body("sign in end point")
}
#[get("/get-user")]
pub async fn get_user(_req: HttpRequest)->impl Responder{
  HttpResponse::Ok().body("get user end point")
}
#[delete("/log-out")]
pub async fn log_out(_req: HttpRequest)->impl Responder{
  HttpResponse::Ok().body("log out end point")
}

pub fn scoped_user(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("user")
      .service(sign_up)
      .service(sign_in)
      .service(get_user)
      .service(log_out)
  );
}