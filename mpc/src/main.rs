use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;

mod config;
mod dto;
mod errors;
mod handlers;
mod routes;
mod services;

use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::init();

    let raw_pool = store::create_db_pool(&config.database_url).await;
    store::init_pool(raw_pool);

    let port   = config.port;
    let config = web::Data::new(config);

    tracing::info!("MPC server starting on port {port}");

    HttpServer::new(move || {
        let cors = Cors::permissive(); 

        App::new()
            .wrap(cors)
            .app_data(config.clone())
            .configure(routes::configure_routes)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
