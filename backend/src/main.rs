use std::io::Result;
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use actix_cors::Cors;
use dotenv::dotenv;
use sqlx::{Pool, Postgres};

mod modules;
mod utils;
mod config;
mod middlewares;

use crate::config::*;
use modules::*;
use store::{create_db_pool, init_pool};

pub async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("You sent a request to no where")
}

pub async fn health_check(pool: web::Data<Pool<Postgres>>) -> impl Responder {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok()
            .json(serde_json::json!({ "status": "ok", "db": "reachable" })),
        Err(_) => HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "status": "error", "db": "unreachable" })),
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = web::Data::new(Config::init());
    let raw_pool = create_db_pool(&config.database_url).await;
    init_pool(raw_pool.clone());
    let pool = web::Data::new(raw_pool);

    store::run_migrations().await.expect("Migration failed");

    tracing::info!("server is running on {}:{}", config.host, config.port);

    let host = config.host.clone();
    let port = config.port;

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .service(
                web::scope("/api")
                    .configure(auth::routes::configure_auth_routes)
                    .configure(user::routes::configure_user_routes)
                    .configure(wallet::routes::configure_wallet_routes)
                    .configure(swap::routes::configure_swap_routes)
                    .configure(link::routes::configure_links_routes)
                    .route("/health", web::get().to(health_check)),
            )
            .default_service(web::to(not_found))
            .app_data(config.clone())
            .app_data(pool.clone())
    })
    .shutdown_timeout(30)
    .bind((host.as_str(), port))?
    .run()
    .await
}