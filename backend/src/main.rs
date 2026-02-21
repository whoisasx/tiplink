use std::io::Result;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use actix_cors::Cors;
use sqlx::migrate::Migrator;

mod modules;
mod utils;
mod config;
mod db;
mod middlewares;
use dotenv::dotenv;
use modules::*;
use crate::config::Config;
use db::*;

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
  let pool=web::Data::new(create_db_pool(&config.database_url).await);
  
  static MIGRATOR:Migrator=sqlx::migrate!("src/db/migrations");
  MIGRATOR.run(pool.as_ref()).await.expect("Migration failed");

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
      .configure(auth::routes::scoped_auth)
      .configure(user::routes::scoped_user)
      .configure(wallet::routes::scoped_wallet)
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