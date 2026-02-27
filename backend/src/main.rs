use std::io::Result;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use actix_cors::Cors;
use dotenv::dotenv;

mod modules;
mod utils;
mod config;
mod middlewares;

use crate::config::*;
use modules::*;
use store::{create_db_pool,init_pool};

pub async fn not_found()->impl Responder{
  HttpResponse::NotFound().body("You sent a request to no where")
}

pub async fn health_check() -> impl Responder{
  HttpResponse::Ok().body("we are up")
}

#[actix_web::main]
async fn main()-> Result<()>{
  dotenv().ok();
  env_logger::init();
  let config=web::Data::new(Config::init());
  let raw_pool = create_db_pool(&config.database_url).await;
  init_pool(raw_pool.clone());
  let pool = web::Data::new(raw_pool);

  store::run_migrations().await.expect("Migration failed");

  println!("server is running on port: 3000");

  HttpServer::new(move || {
    // let cors = Cors::default()
    //     .allow_any_origin()
    //     .allow_any_method()
    //     .allow_any_header()
    //     .supports_credentials();
    let cors=Cors::permissive();
      
    App::new()
    .wrap(cors)
    .service(
      web::scope("/api")
      .configure(auth::routes::configure_auth_routes)
      .configure(user::routes::configure_user_routes)
      .configure(wallet::routes::configure_wallet_routes)
      .configure(swap::routes::configure_swap_routes)
      .configure(link::routes::configure_links_routes)
      .route("/health", web::to(health_check))
    )
    .default_service(web::to(not_found))
    .app_data(config.clone())
    .app_data(pool.clone())
  })
  .bind(("127.0.0.1", 3000))?
  .run()
  .await
}