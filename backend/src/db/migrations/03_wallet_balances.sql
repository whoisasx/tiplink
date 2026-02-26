CREATE TABLE wallet_balances(
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id             UUID NOT NULL REFERENCES users(id),
  wallet_pubkey       TEXT NOT NULL,
  mint                TEXT NOT NULL,  --'SOL' or SPL for mint address
  symbol              TEXT NOT NULL,  --SOL, USDC , USDT
  decimals            INT NOT NULL,
  raw_amount          BIGINT NOT NULL,  -- in base units (lamports / token smallest unit)
  ui_amount           NUMERIC(28,9),    -- human readable (raw / 10^decimals)
  usd_value           NUMERIC(18,2),    -- filled by indexer using price feed
  last_synced_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_wallet_balances_user_id ON wallet_balances(user_id);
CREATE INDEX idx_wallet_balances_pubkey ON wallet_balances(wallet_pubkey);