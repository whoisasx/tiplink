use std::{io::Result};
use actix_web::{App, HttpServer, web};

mod routes;
mod utils;
use routes::*;

#[actix_web::main]
async fn main()-> Result<()>{
  dotenv::dotenv().ok();
  env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

  HttpServer::new(move || {
    App::new()
    .service(
      web::scope("/api")
      .configure(scoped_user)
      .configure(scoped_solana)
      .route("/health", web::to(health_check))
    )
    .default_service(web::to(not_found))
  })
  .bind(("127.0.0.1", 3000))?
  .run()
  .await
}