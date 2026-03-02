use actix_web::{App, HttpResponse, HttpServer, Responder, get};
use store::{create_db_pool, init_pool};

pub mod webhooks;
use webhooks::*;

#[get("/")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("indexer ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let port: u16 = std::env::var("INDEXER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("INDEXER_PORT must be a valid port number");

    let raw_pool = create_db_pool(&database_url).await;
    init_pool(raw_pool);

    let solana_rpc_url = std::env::var("SOLANA_RPC_URL")
        .expect("SOLANA_RPC_URL must be set");
    let rpc_url = actix_web::web::Data::new(solana_rpc_url);

    tracing::info!(port, "indexer listening");

    HttpServer::new(move || {
        App::new()
            .app_data(rpc_url.clone())
            .service(health)
            .service(handle_hooks)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}