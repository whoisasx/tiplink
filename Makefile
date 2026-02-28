# ============================================================
# tiplink — dev task runner
# Usage: make <target>
# ============================================================

include .env
export

# make's `include` preserves surrounding quotes in values — strip them
DATABASE_URL        := $(subst ",,$(DATABASE_URL))
DIRECT_DATABASE_URL := $(subst ",,$(DIRECT_DATABASE_URL))

# ── Database ─────────────────────────────────────────────────

# Apply all pending migrations (same as what main.rs does at startup)
migrate:
	cargo sqlx migrate run --source store/migrations

# Roll back the latest migration
migrate-revert:
	cargo sqlx migrate revert --source store/migrations

# Show migration status
migrate-status:
	cargo sqlx migrate info --source store/migrations

# Regenerate the .sqlx offline query cache (uses DIRECT_DATABASE_URL, not pooler)
# Run this after adding/changing any sqlx::query_as!() macro
sqlx-prepare:
	DATABASE_URL=$(DIRECT_DATABASE_URL) cargo sqlx prepare --workspace

# ── Build ────────────────────────────────────────────────────

# Build all workspace crates (also generates the proc-macro dylibs for rust-analyzer)
build:
	cargo build --workspace

# Build in release mode
build-release:
	cargo build --workspace --release

# ── Run ──────────────────────────────────────────────────────

dev:
	cd backend && cargo watch -w src -w ../store/src -w ../.env -x run

# ── Checks ───────────────────────────────────────────────────

check:
	cargo check --workspace

lint:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all

.PHONY: migrate migrate-revert migrate-status sqlx-prepare build build-release dev check lint fmt
