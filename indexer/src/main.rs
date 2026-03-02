use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use store::{create_db_pool, init_pool};

mod config;
pub mod webhooks;
use config::Config;
use webhooks::*;

#[get("/")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("indexer ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::init();

    let raw_pool = create_db_pool(&config.database_url).await;
    init_pool(raw_pool);

    let rpc_url = web::Data::new(config.solana_rpc_url.clone());
    let host    = config.host.clone();
    let port    = config.port;

    tracing::info!(host = %host, port, "indexer listening");

    HttpServer::new(move || {
        App::new()
            .app_data(rpc_url.clone())
            .service(health)
            .service(handle_hooks)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}