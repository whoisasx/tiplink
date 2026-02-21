CREATE TABLE wallet_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID UNIQUE NOT NULL REFERENCES users(id),
    pubkey          TEXT UNIQUE NOT NULL,
    shard_index     INT NOT NULL,
    encrypted_share   TEXT NOT NULL,          -- reference ID to fetch Share A from vault/KMS
    -- NOT the actual share bytes
    status          TEXT DEFAULT 'active',  -- 'active', 'rotating', 'revoked'
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    signature       TEXT UNIQUE,            -- Solana tx signature, null if pending
    type            TEXT NOT NULL,          -- 'send', 'receive', 'swap', 'fiat_deposit', 'fiat_withdraw'
    status          TEXT DEFAULT 'pending', -- 'pending', 'confirmed', 'failed'
    amount          BIGINT NOT NULL,        -- in base units (lamports / token decimals)
    mint            TEXT,                   -- null = SOL, otherwise SPL mint address
    from_address    TEXT,
    to_address      TEXT,
    fee             BIGINT,
    metadata        JSONB,                  -- flexible: memo, swap details, fiat reference, etc.
    confirmed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_signature ON transactions(signature);
CREATE INDEX idx_transactions_status ON transactions(status);