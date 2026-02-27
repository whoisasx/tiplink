use std::sync::OnceLock;
use sqlx::{Pool, migrate::MigrateError, postgres::{PgPoolOptions, Postgres}};

static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();

pub async fn create_db_pool(database_url: &str) -> Pool<Postgres> {
  PgPoolOptions::new()
    .max_connections(20)
    .connect(database_url)
    .await
    .expect("Failed to create database pool")
}

pub fn init_pool(pool: Pool<Postgres>) {
  DB_POOL.set(pool).expect("DB pool already initialised");
}

pub fn pool() -> &'static Pool<Postgres> {
  DB_POOL.get().expect("DB pool not initialised — call init_pool first")
}

pub async fn run_migrations() -> Result<(), MigrateError> {
  sqlx::migrate!("./migrations")
    .run(pool())
    .await
}
