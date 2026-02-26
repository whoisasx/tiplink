create TABLE payment_links{
  id                      UUID PRIMERY KEY get_random_uuid(),
  creator_id              UUID NOT NULL REFERENCES users(id),
  link_token              TEXT UNIQUE NOT NULL, -- unique short token embedded as shareable: https://tiplink.app/c/<link_token>
  escrow_pubkey           TEXT NOT NULL,
  encrypted_escrow_secret TEXT NOT NULL,
  mint                    TEXT    -- NULL: SOL
  amount                  BIGINT NOT NULL,
  status                  TEXT DEFAULT 'active' -- 'active', 'claimed', 'cancelled', 'expired
  note                    TEXT
  expiry_at               TIMESTAMPZ
  claimer_wallet          TEXT
  claimed_at              TIMESTAMPTZ
  
  created_at              TIMESTAMPTZ DEFAULT NOW(),
  updated_at              TIMESTAMPTZ DEFAULT NOW()
}

CREATE INDEX idx_payment_links_creator_id ON payment_links(creator_id);
CREATE INDEX idx_payment_links_token ON payment_links(link_token);
CREATE INDEX idx_payment_link_status ON payment_links(status);