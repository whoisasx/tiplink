use actix_web::{HttpResponse, Responder, delete, get, post, web};

#[post("/sign-up")]
pub async fn sign_up()->impl Responder{
  HttpResponse::Ok().body("sign up end point")
}
#[post("/sign-in")]
pub async fn sign_in()->impl Responder{
  HttpResponse::Ok().body("sign in end point")
}
#[get("/get-user")]
pub async fn get_user()->impl Responder{
  HttpResponse::Ok().body("get user end point")
}
#[delete("/log-out")]
pub async fn log_out()->impl Responder{
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