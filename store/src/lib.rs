// =============================================================================
// store/src/lib.rs  —  SHARED DATABASE LAYER
// =============================================================================
//
// PURPOSE
// -------
// `store` is a Cargo library crate shared by `backend` and `indexer`.
// It owns:
//   - the pool singleton (create, init, get)
//   - all row-level DTOs (one struct per DB table)
//   - all query functions (one file per table)
//
// Neither `backend` nor `indexer` touches sqlx directly.
// They import functions from this crate and call them.
//
// =============================================================================
// STEP 1 — Fix store/Cargo.toml  (sqlx needs postgres features)
// =============================================================================
//
// The current Cargo.toml has:
//   sqlx = "0.8.6"
//
// Change it to:
//   sqlx = { version = "0.8.6", features = [
//     "runtime-tokio", "postgres", "chrono", "uuid", "tls-rustls"
//   ]}
//
// Also make sure these are present (they already are):
//   chrono  = { version = "0.4", features = ["serde"] }
//   tokio   = { version = "1",   features = ["full"] }
//   uuid    = { version = "1",   features = ["v4"] }
//   serde   = { version = "1",   features = ["derive"] }   ← ADD this
//   serde_json = "1"                                        ← ADD this (for JSONB fields)
//
// =============================================================================
// STEP 2 — Add `store` as a dependency in backend/Cargo.toml
// =============================================================================
//
// The workspace Cargo.toml already lists all four crates as members,
// so Cargo knows about `store`. Just add it as a path dependency in backend:
//
//   # backend/Cargo.toml  [dependencies]
//   store = { path = "../store" }
//
// Same for indexer when you get to it:
//   # indexer/Cargo.toml  [dependencies]
//   store = { path = "../store" }
//
// =============================================================================
// STEP 3 — File layout inside store/src/
// =============================================================================
//
//   store/src/
//     lib.rs          ← this file — re-exports every sub-module
//     pool.rs         ← OnceLock singleton: create_db_pool, init_pool, pool()
//                        move the contents of backend/src/db/handlers.rs here
//     dto.rs          ← all row-level structs, one per table
//     users.rs        ← query fns for `users` + `refresh_tokens`
//     wallet_keys.rs  ← query fns for `wallet_keys`
//     transactions.rs ← query fns for `transactions`
//     balances.rs     ← query fns for `wallet_balances`
//     swap_quotes.rs  ← query fns for `swap_quotes`
//     payment_links.rs← query fns for `payment_links`
//
// =============================================================================
// STEP 4 — pool.rs (move from backend/src/db/handlers.rs)
// =============================================================================
//
//   use std::sync::OnceLock;
//   use sqlx::{Pool, postgres::{PgPoolOptions, Postgres}};
//
//   static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();
//
//   pub async fn create_db_pool(database_url: &str) -> Pool<Postgres> {
//     PgPoolOptions::new()
//       .max_connections(10)
//       .connect(database_url)
//       .await
//       .expect("Failed to create database pool")
//   }
//
//   pub fn init_pool(pool: Pool<Postgres>) {
//     DB_POOL.set(pool).expect("DB pool already initialised");
//   }
//
//   pub fn pool() -> &'static Pool<Postgres> {
//     DB_POOL.get().expect("DB pool not initialised — call init_pool first")
//   }
//
// =============================================================================
// STEP 5 — dto.rs  (move + clean up backend/src/db/dto.rs)
// =============================================================================
//
//   These structs map column-for-column to each table.
//   They are NOT the API response types — those stay in modules/*/dto.rs.
//
//   pub struct UserRow {
//     pub id: String, pub google_sub: String, pub email: String,
//     pub display_name: Option<String>, pub avatar_url: Option<String>,
//     pub wallet_pubkey: Option<String>, pub is_active: bool,
//     pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
//   }
//
//   pub struct RefreshTokenRow {
//     pub id: String, pub user_id: String, pub token_hash: String,
//     pub expires_at: DateTime<Utc>, pub revoked: bool,
//     pub revoked_at: Option<DateTime<Utc>>, pub user_agent: Option<String>,
//     pub ip_address: Option<String>, pub created_at: DateTime<Utc>,
//   }
//
//   pub struct WalletKeyRow {
//     pub id: String, pub user_id: String, pub pubkey: String,
//     pub shard_index: i32, pub encrypted_share: String,
//     pub status: String, pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
//   }
//
//   pub struct TransactionRow {
//     pub id: String, pub user_id: String, pub signature: Option<String>,
//     pub txn_type: String, pub status: String, pub amount: i64,
//     pub mint: Option<String>, pub from_address: Option<String>,
//     pub to_address: Option<String>, pub fee: Option<i64>,
//     pub metadata: Option<serde_json::Value>,
//     pub confirmed_at: Option<DateTime<Utc>>,
//     pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
//   }
//
//   pub struct WalletBalanceRow {
//     pub id: String, pub user_id: String, pub wallet_pubkey: String,
//     pub mint: String, pub symbol: String, pub decimals: i32,
//     pub raw_amount: i64, pub ui_amount: Option<f64>, pub usd_value: Option<f64>,
//     pub last_synced_at: DateTime<Utc>,
//   }
//
//   pub struct SwapQuoteRow {
//     pub id: String, pub user_id: String,
//     pub input_mint: String, pub output_mint: String,
//     pub quoted_amount: i64, pub slippage_bps: i32,
//     pub price_impact_pct: Option<f64>,
//     pub route_plan: serde_json::Value,
//     pub unsigned_tx: String, pub expires_at: DateTime<Utc>,
//     pub status: String, pub transaction_id: Option<String>,
//     pub created_at: DateTime<Utc>,
//   }
//
//   pub struct PaymentLinkRow {
//     pub id: String, pub creator_id: String,
//     pub link_token: String, pub escrow_pubkey: String,
//     pub encrypted_escrow_secret: String,
//     pub mint: Option<String>, pub amount: i64, pub status: String,
//     pub note: Option<String>, pub expiry_at: Option<DateTime<Utc>>,
//     pub claimer_wallet: Option<String>, pub claimed_at: Option<DateTime<Utc>>,
//     pub created_at: DateTime<Utc>, pub updated_at: DateTime<Utc>,
//   }
//
// =============================================================================
// STEP 6 — Query functions (same naming convention as backend/db plan)
// =============================================================================
//
//   Every function:
//     1. calls  crate::pool::pool()  to get the pool reference
//     2. runs a sqlx::query_as! or sqlx::query! macro
//     3. returns a Result<T> or Option<T> — NO panics, NO business logic
//
//   Naming:
//     find_*   → Option<Row>    (single row, None if not found)
//     list_*   → Vec<Row>       (multiple rows, with filter/pagination args)
//     insert_* → Result<Row>    (returns created row)
//     update_* → Result<bool>   (true = row was affected)
//     delete_* → Result<bool>
//     upsert_* → Result<Row>
//
// =============================================================================
// STEP 7 — How backend uses store after the migration
// =============================================================================
//
//   BEFORE (current):
//     backend/src/db/handlers.rs   — owns pool singleton
//     backend/src/db/users.rs      — owns query fns
//     backend/src/modules/auth/services.rs  — imports from crate::db::*
//
//   AFTER:
//     store owns everything above
//     backend/src/db/ is DELETED entirely
//
//   In backend/src/main.rs:
//     // before: use db::*;
//     use store::{create_db_pool, init_pool};
//
//     let raw_pool = create_db_pool(&config.database_url).await;
//     store::init_pool(raw_pool.clone());           // init the singleton in store
//     let pool = web::Data::new(raw_pool);          // still needed for sqlx migrations
//
//   In backend/src/modules/auth/services.rs:
//     // before: use crate::db::users::{find_user_by_google_sub, upsert_user};
//     use store::users::{find_user_by_google_sub, upsert_user};
//
//   In backend/src/middlewares/auth.rs  (or wherever pool was used directly):
//     // before: use crate::db::pool;
//     use store::pool;
//
//   Nothing else in backend changes — service logic, handlers, routes all stay.
//
// =============================================================================
// STEP 8 — Remove the now-redundant db/ folder from backend
// =============================================================================
//
//   Once every import in backend is pointing at `store::*`:
//     1. Delete backend/src/db/handlers.rs
//     2. Delete backend/src/db/users.rs
//     3. Delete backend/src/db/dto.rs
//     4. Delete backend/src/db/services.rs  (was empty)
//     5. Delete backend/src/db/mod.rs
//     6. Remove `mod db;` and `use db::*;` from backend/src/main.rs
//
// =============================================================================
// MODULE DECLARATIONS  (uncomment as you implement each file)
// =============================================================================

pub mod pool;
pub mod dto;
pub mod users;
pub mod wallet_keys;
pub mod transactions;
pub mod balances;
pub mod swap_quotes;
pub mod payment_links;

pub use pool::{create_db_pool, init_pool, pool};
