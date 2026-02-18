use actix_web::{HttpResponse, Responder};

pub async fn not_found()->impl Responder{
  HttpResponse::NotFound().body("You sent a request to no where")
}

pub async fn health_check() -> impl Responder{
  HttpResponse::Ok().body("we are up")
}