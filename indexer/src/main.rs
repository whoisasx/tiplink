use actix_web::{App, HttpResponse, HttpServer, Responder, get};

pub mod webhooks;
use webhooks::*;

#[get("/")]
async fn manuall_hello()->impl Responder{
  HttpResponse::Ok().body("hello from 8080")
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
  tracing_subscriber::fmt()
    .with_env_filter("info")
    .init();

  tracing::info!("server is running on port: 8080");
  HttpServer::new(||{
    App::new()
      .service(manuall_hello)
      .service(handle_hooks)
  })
  .bind(("127.0.0.1",8080))?
  .run()
  .await
}