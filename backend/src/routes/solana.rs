use actix_web::{HttpResponse, Responder, get, web};

#[get("/")]
pub async fn hello_solana()->impl Responder{
  HttpResponse::Ok().body("hello from solana")
}

pub fn scoped_solana(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("/solana")
    .service(hello_solana)
  );
}