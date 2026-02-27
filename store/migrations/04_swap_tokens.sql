CREATE TABLE swap_quotes (
  id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id               UUID NOT NULL REFERENCES users(id),
  input_mint            TEXT NOT NULL,
  output_mint           TEXT NOT NULL,
  quoted_amount         BIGINT NOT NULL,
  slippage_bps          INT NOT NULL DEFAULT 50,
  price_impact_pct      NUMERIC(10,4),
  route_plan            JSONB NOT NULL,
  unsigned_tx           TEXT NOT NULL,
  expires_at            TIMESTAMPTZ NOT NULL,
  status                TEXT DEFAULT 'pending', -- 'pending', 'executed', 'expired', 'failed'
  transaction_id        UUID REFERENCES transactions(id),
  created_at            TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_swap_quotes_user_id ON swap_quotes(user_id);
CREATE INDEX idx_swap_quotes_status ON swap_quotes(status);