use actix_web::{HttpResponse, Responder, get, web};

#[get("/me")]
pub async fn get_user() -> impl Responder {
  HttpResponse::Ok().body("hi there")
}

pub fn scoped_user(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("user")
      .service(get_user)
  );
}