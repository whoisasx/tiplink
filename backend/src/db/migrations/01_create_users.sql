CREATE TABLE users (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  google_sub      TEXT UNIQUE NOT NULL,   -- Google's permanent user ID
  email           TEXT UNIQUE NOT NULL,
  display_name    TEXT,
  avatar_url      TEXT,
  wallet_pubkey   TEXT UNIQUE,            -- set after MPC generates wallet
  is_active       BOOLEAN DEFAULT TRUE,
  created_at      TIMESTAMPTZ DEFAULT NOW(),
  updated_at      TIMESTAMPTZ DEFAULT NOW()
);
CREATE TABLE refresh_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash      TEXT UNIQUE NOT NULL,   -- bcrypt/sha256 hash of the token
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked         BOOLEAN DEFAULT FALSE,
    revoked_at      TIMESTAMPTZ,
    user_agent      TEXT,                   -- which device/browser
    ip_address      TEXT,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);