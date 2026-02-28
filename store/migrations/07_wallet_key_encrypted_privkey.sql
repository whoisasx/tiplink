-- Make the original MPC-shard columns nullable (they are no longer required
-- now that the MPC server stores the full encrypted private key directly).
ALTER TABLE wallet_keys ALTER COLUMN shard_index    DROP NOT NULL;
ALTER TABLE wallet_keys ALTER COLUMN encrypted_share DROP NOT NULL;

-- Stores the AES-256-GCM encrypted ed25519 private key (64 bytes).
-- Encrypted with: key = SHA-256(master_secret + ":privkey_enc")
--                nonce = SHA-256(user_id)[0..12]
-- Encoded as hex. Only the MPC server (which holds master_secret) can decrypt.
ALTER TABLE wallet_keys ADD COLUMN encrypted_private_key TEXT;
