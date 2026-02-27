ALTER TABLE wallet_balances ADD CONSTRAINT uq_wallet_balances_pubkey_mint UNIQUE (wallet_pubkey, mint);
