# tiplink-server

A Rust-based backend monorepo for TipLink — a Solana-powered payment link platform. Users can create shareable payment links backed by escrow wallets, swap tokens via Jupiter, and manage custodial wallets derived through an MPC key server.

---

## Architecture

The workspace is split into four crates, each responsible for a distinct concern:

```
tiplink-server/
├── backend/     # Main REST API (Actix-web)
├── indexer/     # Helius webhook listener (Solana tx indexer)
├── mpc/         # Key-management server (wallet derive + sign)
└── store/       # Shared DB library (SQLx, migrations, query fns)
```

```
┌─────────────┐    REST/JSON    ┌─────────────┐
│  tiplink-web│ ──────────────► │   backend   │ ──► PostgreSQL
└─────────────┘                 └──────┬──────┘
                                       │ internal HTTP
                                       ▼
                                  ┌─────────┐
                                  │   mpc   │ ──► Solana RPC
                                  └─────────┘
                                       ▲
                               ┌───────┴──────┐
                               │   indexer    │ ◄── Helius webhooks
                               └──────────────┘
```

---

## Crates

### `backend` — Main API Server

Actix-web HTTP server that exposes the full REST API under the `/api` prefix.

| Scope  | Method   | Endpoint                        | Auth | Description                    |
| ------ | -------- | ------------------------------- | ---- | ------------------------------ |
| auth   | `GET`    | `/api/auth/google`              | —    | Initiate Google OAuth flow     |
| auth   | `GET`    | `/api/auth/callback/google`     | —    | OAuth callback, issues JWT     |
| auth   | `POST`   | `/api/auth/refresh`             | —    | Rotate access + refresh tokens |
| auth   | `POST`   | `/api/auth/logout`              | JWT  | Revoke refresh token           |
| user   | `GET`    | `/api/user/me`                  | JWT  | Get authenticated user profile |
| user   | `PATCH`  | `/api/user/me`                  | JWT  | Update display name / avatar   |
| wallet | `GET`    | `/api/wallet/balances`          | JWT  | Fetch all token balances       |
| wallet | `GET`    | `/api/wallet/balance/:mint`     | JWT  | Fetch balance for one token    |
| wallet | `GET`    | `/api/wallet/transactions`      | JWT  | Paginated transaction history  |
| wallet | `GET`    | `/api/wallet/transactions/:sig` | JWT  | Single transaction detail      |
| wallet | `POST`   | `/api/wallet/send`              | JWT  | Build + submit a transfer      |
| wallet | `GET`    | `/api/wallet/estimate-fee`      | JWT  | Estimate transaction fee       |
| swap   | `GET`    | `/api/swap/quote`               | JWT  | Get Jupiter DEX quote          |
| swap   | `POST`   | `/api/swap/execute`             | JWT  | Execute swap via Jupiter       |
| swap   | `GET`    | `/api/swap/history`             | JWT  | Swap history for user          |
| link   | `POST`   | `/api/link/create`              | JWT  | Create a payment link          |
| link   | `GET`    | `/api/link/my`                  | JWT  | List creator's links           |
| link   | `GET`    | `/api/link/:token`              | —    | Look up a link (public)        |
| link   | `POST`   | `/api/link/:token/claim`        | —    | Claim a payment link           |
| link   | `DELETE` | `/api/link/:token/cancel`       | JWT  | Cancel an active link          |
| —      | `GET`    | `/api/health`                   | —    | DB health check                |

**Key dependencies:** `actix-web`, `actix-cors`, `sqlx`, `jsonwebtoken`, `oauth2`, `reqwest`, `serde`, `uuid`, `tracing`

---

### `mpc` — Key Management Server

A separate Actix-web service responsible for deterministic wallet derivation and on-chain signing. It never stores raw private keys in the database — keys are derived at request time from a master secret via `HMAC-SHA256` and then AES-256-GCM encrypted at rest.

**Responsibilities:**

- Derive a user's Solana keypair from `HMAC(master_secret, user_id)`
- Encrypt the derived private key with `AES-256-GCM` before persisting
- Build and sign Solana transactions (SOL transfers, SPL token transfers, ATA creation)
- Used exclusively by `backend` via internal HTTP — never exposed to the public internet

---

### `indexer` — Transaction Indexer

A lightweight Actix-web server that receives real-time webhook events from [Helius](https://helius.dev) and records incoming transactions against known wallet addresses in the database.

**Responsibilities:**

- Accept Helius webhook `POST` payloads
- Parse and persist transaction data to the `wallet_transactions` table
- Update wallet balances when relevant transfers are detected

---

### `store` — Shared Database Library

A library crate consumed by `backend`, `mpc`, and `indexer`. Provides:

- Connection pool initialisation (`create_db_pool`, `init_pool`, `pool`)
- `sqlx` migration runner (`run_migrations`)
- Typed query functions for every table: `users`, `wallet_keys`, `transactions`, `balances`, `swap_quotes`, `payment_links`

#### Database Schema (7 migrations)

| Migration                             | Table / Change                     |
| ------------------------------------- | ---------------------------------- |
| `01_create_users.sql`                 | `users`, `refresh_tokens`          |
| `02_wallet_txn.sql`                   | `wallet_transactions`              |
| `03_wallet_balances.sql`              | `wallet_balances`                  |
| `04_swap_tokens.sql`                  | `swap_quotes`                      |
| `05_payment_links.sql`                | `payment_links`                    |
| `06_wallet_balances_unique.sql`       | Unique constraint on balances      |
| `07_wallet_key_encrypted_privkey.sql` | `wallet_keys` encrypted key column |

---

## Prerequisites

| Tool          | Version                                                                   |
| ------------- | ------------------------------------------------------------------------- |
| Rust          | stable (2024 edition)                                                     |
| `cargo-watch` | `cargo install cargo-watch`                                               |
| `sqlx-cli`    | `cargo install sqlx-cli --no-default-features --features postgres,rustls` |
| PostgreSQL    | Neon (serverless) or any Postgres 14+ instance                            |

---

## Environment Variables

Copy `.env.example` to `.env` and fill in every value:

```dotenv
# SQLx offline query cache (no DB needed at compile time)
SQLX_OFFLINE=true

# Google OAuth
GOOGLE_OAUTH_CLIENT_ID=
GOOGLE_OAUTH_CLIENT_SECRET=

# Callback URL must match the one registered in Google Cloud Console
REDIRECT_URL=http://localhost:3000/api/auth/callback/google

# JWT
JWT_SECRET=
TOKEN_EXPIRED_IN=60m
TOKEN_MAXAGE=60

# CORS / origins
CLIENT_ORIGIN=http://localhost:5173
BASE_URL=http://localhost:3000

# PostgreSQL (Neon or compatible)
DATABASE_URL=postgresql://user:pass@host/dbname?sslmode=require
DIRECT_DATABASE_URL=postgresql://user:pass@host/dbname?sslmode=require

# CSRF protection
RANDOM_CSRF_STRING=

# Solana
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# Internal MPC service URL
MPC_SERVER_URL=http://localhost:4000
MPC_MASTER_SECRET=

# Helius (optional — for webhook-based indexing)
HELIUS_API_KEY=
HELIUS_WEBHOOK_ID=

# Escrow wallet used to fund / receive payment links
ESCROW_PUBLIC_KEY=
ESCROW_PRIVATE_KEY=
ESCROW_HMAC_SECRET=

# Jupiter DEX
JUPITER_QUOTE_URL=https://quote-api.jup.ag/v6/quote
JUPITER_SWAP_URL=https://quote-api.jup.ag/v6/swap
SOL_MINT=So11111111111111111111111111111111111111112
SOLANA_BASE_FEE_LAMPORTS=5000
```

---

## Getting Started

### 1. Install dependencies

```bash
cargo build --workspace
```

### 2. Run database migrations

```bash
make migrate
```

### 3. Start the development servers

In separate terminals (or use a process manager):

```bash
# Main API — auto-reloads on source changes
make dev

# MPC key server
cd mpc && cargo watch -w src -x run

# Indexer
cd indexer && cargo watch -w src -x run
```

The API will be available at `http://localhost:3000`.

---

## Makefile Targets

| Target                | Description                                |
| --------------------- | ------------------------------------------ |
| `make migrate`        | Apply pending migrations                   |
| `make migrate-revert` | Roll back the latest migration             |
| `make migrate-status` | Show current migration state               |
| `make sqlx-prepare`   | Regenerate the `.sqlx` offline query cache |
| `make build`          | Build all workspace crates (debug)         |
| `make build-release`  | Build all workspace crates (release)       |
| `make dev`            | Run backend with `cargo-watch`             |
| `make check`          | Type-check the whole workspace             |
| `make lint`           | Run `clippy` with `-D warnings`            |
| `make fmt`            | Format all code with `rustfmt`             |

---

## Authentication Flow

```
Browser                 backend                   Google OAuth
  │                        │                            │
  │  GET /api/auth/google  │                            │
  │ ──────────────────────►│ ── redirect ──────────────►│
  │                        │                            │
  │ ◄── redirect (code) ───────────────────────────────│
  │                        │                            │
  │  GET /callback?code=…  │                            │
  │ ──────────────────────►│ ── exchange code ─────────►│
  │                        │ ◄── id_token / profile ────│
  │                        │
  │                        │  upsert user in DB
  │                        │  sign JWT access token (60 min)
  │                        │  store refresh token hash in DB
  │ ◄── Set-Cookie (JWT) ──│
```

Access tokens are short-lived (60 min). Clients call `POST /api/auth/refresh` with the refresh token cookie to obtain a new pair without re-authenticating.

---

## Payment Links

A payment link encapsulates a one-time SOL or SPL-token transfer:

1. **Create** — the creator specifies an amount, optional note, expiry, and token mint. The backend generates a unique `link_token`, funds an escrow account, and stores the encrypted escrow secret.
2. **Claim** — anyone with the link token calls `POST /api/link/:token/claim`. The backend verifies the link is `active`, transfers funds from the escrow to the claimer's wallet, and marks the link `claimed`.
3. **Cancel** — the creator can `DELETE /api/link/:token/cancel` to reclaim escrowed funds.

---

## Token Swaps

Swaps are powered by the [Jupiter Aggregator](https://jup.ag):

1. `GET /api/swap/quote` — proxies a quote request to the Jupiter Quote API.
2. `POST /api/swap/execute` — fetches the best route, builds the swap transaction via the Jupiter Swap API, signs it through the MPC service, and submits it to the Solana network.

---

## Project Structure

```
tiplink-server/
├── .cargo/config.toml          # Cargo configuration
├── .env.example                # Environment variable template
├── .sqlx/                      # Offline SQLx query cache
├── Cargo.toml                  # Workspace manifest
├── Makefile                    # Dev task runner
│
├── backend/src/
│   ├── main.rs                 # Server bootstrap, route registration
│   ├── config.rs               # Typed config from env vars
│   ├── state.rs                # Shared application state
│   ├── middlewares/
│   │   ├── auth.rs             # JWT validation middleware
│   │   └── mod.rs
│   ├── modules/
│   │   ├── auth/               # Google OAuth + JWT handlers
│   │   ├── user/               # User profile handlers
│   │   ├── wallet/             # Balance, transfer, fee handlers
│   │   ├── swap/               # Jupiter swap handlers
│   │   ├── link/               # Payment link handlers
│   │   └── dto.rs              # Shared request/response types
│   └── utils/                  # Helpers (hashing, errors, …)
│
├── indexer/src/
│   ├── main.rs                 # Indexer HTTP server bootstrap
│   ├── config.rs
│   └── webhooks/               # Helius webhook handlers
│
├── mpc/src/
│   ├── main.rs                 # MPC server bootstrap
│   ├── config.rs
│   ├── services.rs             # Key derivation + Solana signing logic
│   ├── handlers.rs
│   ├── routes.rs
│   ├── dto.rs
│   ├── errors.rs
│   └── middlewares.rs
│
└── store/
    ├── src/
    │   ├── lib.rs              # Public API re-exports
    │   ├── pool.rs             # DB pool management + migrations
    │   ├── users.rs
    │   ├── wallet_keys.rs
    │   ├── transactions.rs
    │   ├── balances.rs
    │   ├── swap_quotes.rs
    │   └── payment_links.rs
    └── migrations/             # SQL migration files (numbered)
```
