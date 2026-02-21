use sqlx::{Pool, postgres::{Postgres, PgPoolOptions}};
pub mod dto;
use dto::*;

pub async fn create_db_pool(database_url: &String)-> Pool<Postgres>{
  PgPoolOptions::new()
  .max_connections(10)
  .connect(database_url)
  .await
  .expect("Failed to create database pool")
}