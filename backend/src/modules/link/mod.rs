// =============================================================================
// MODULE: link
// PURPOSE: TipLink-style payment links — create a shareable link that holds
//          SOL / SPL tokens, claimable by anyone with the URL.
//          The link holds funds in an escrow-style PDA (on-chain) OR a
//          server-generated ephemeral keypair (custodial approach).
//          All creation/management routes require JWT auth.
//          Claim route is PUBLIC (no auth required — anyone with the link claims).
// =============================================================================

// -----------------------------------------------------------------------------
// DB TABLES NEEDED (new table required)
// -----------------------------------------------------------------------------
//
// CREATE TABLE payment_links (
//   id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//   creator_id     UUID NOT NULL REFERENCES users(id),
//
//   -- The unique short token embedded in the shareable URL
//   -- e.g. https://tiplink.app/c/<link_token>
//   link_token     TEXT UNIQUE NOT NULL,
//
//   -- Escrow wallet that holds the funds for this link
//   escrow_pubkey  TEXT NOT NULL,
//
//   -- Encrypted private key of the escrow keypair (server-side custodial)
//   -- OR a reference to vault/KMS entry if using external key management
//   encrypted_escrow_secret  TEXT NOT NULL,
//
//   mint           TEXT,           -- NULL = SOL, otherwise SPL mint address
//   amount         BIGINT NOT NULL, -- amount locked in base units
//
//   status         TEXT DEFAULT 'active',
//                  -- 'active'   : funded, not yet claimed
//                  -- 'claimed'  : already claimed
//                  -- 'cancelled': creator cancelled and reclaimed funds
//                  -- 'expired'  : past expiry_at and unclaimed
//
//   -- Optional fields
//   note           TEXT,           -- optional message from creator
//   expiry_at      TIMESTAMPTZ,    -- NULL = no expiry
//
//   claimer_wallet TEXT,           -- filled once claimed (could be unregistered user)
//   claimed_at     TIMESTAMPTZ,
//
//   created_at     TIMESTAMPTZ DEFAULT NOW(),
//   updated_at     TIMESTAMPTZ DEFAULT NOW()
// );
//
// CREATE INDEX idx_payment_links_creator_id  ON payment_links(creator_id);
// CREATE INDEX idx_payment_links_link_token  ON payment_links(link_token);
// CREATE INDEX idx_payment_links_status      ON payment_links(status);
//
// NOTE: Each link creation should also create a row in `transactions` table
//   type='send', status='confirmed', from_address=user_wallet,
//   to_address=escrow_pubkey, metadata={"link_id": "<uuid>", "note": "..."}
//
// NOTE: Each link claim should create a row in `transactions` table
//   type='receive', status='confirmed', from_address=escrow_pubkey,
//   to_address=claimer_wallet, metadata={"link_id": "<uuid>"}
// -----------------------------------------------------------------------------

// =============================================================================
// ROUTE MAP
// =============================================================================

// -----------------------------------------------------------------------------
// 1. POST /link/create
// -----------------------------------------------------------------------------
// PURPOSE  : Create a new payment link. Transfers funds from user's wallet
//            into a freshly generated escrow keypair. Returns the shareable URL.
//
// AUTH     : Required (JWT Bearer)
//
// REQUEST BODY (JSON):
//   {
//     "amount":     u64,     -- amount in base units
//     "mint":       String?, -- null/omit = SOL, else SPL mint address
//     "note":       String?, -- optional message shown to claimer
//     "expiry_at":  String?  -- ISO 8601 datetime, null = no expiry
//   }
//
// RESPONSE (201):
//   {
//     "link_id":     String,  -- UUID
//     "link_token":  String,  -- short random token
//     "link_url":    String,  -- full shareable URL: "{base_url}/c/{link_token}"
//     "escrow_pubkey": String,
//     "amount":      u64,
//     "mint":        String?,
//     "note":        String?,
//     "expiry_at":   String?,
//     "status":      "active",
//     "created_at":  String
//   }
//
// ERRORS:
//   400 - invalid amount / mint / expiry
//   401 - unauthorized
//   402 - insufficient balance
//   500 - keypair generation or transfer failure
//
// WORK TO DO:
//   [ ] Define CreateLinkRequest dto  (amount, mint, note, expiry_at)
//   [ ] Define CreateLinkResponse dto (link_id, link_url, escrow_pubkey, ...)
//   [ ] In handler: extract JWT claims (user_id, wallet_pubkey)
//   [ ] In service:
//         a. Validate amount > 0, expiry_at is future if provided
//         b. Check user's on-chain balance >= amount + estimated fee
//         c. Generate a new ephemeral keypair (escrow)
//         d. Build + sign + broadcast Solana transfer tx
//              FROM user_wallet TO escrow_pubkey for {amount} of {mint}
//              Use MPC signing for user_wallet (like wallet/send route)
//         e. Encrypt & store the escrow keypair secret (in encrypted_escrow_secret)
//         f. Generate a unique link_token (nanoid, 12-16 chars)
//         g. Insert into `payment_links` table (status='active')
//         h. Insert into `transactions` table (type='send', ...)
//         i. Return the full response with link URL
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 2. GET /link/my
// -----------------------------------------------------------------------------
// PURPOSE  : List all payment links created by the authenticated user.
//
// AUTH     : Required (JWT Bearer)
//
// QUERY PARAMS:
//   - page   : u32   (default 1)
//   - limit  : u32   (default 20)
//   - status : String?  -- filter by 'active'|'claimed'|'cancelled'|'expired'|'all'
//
// RESPONSE (200):
//   {
//     "pagination": { page, limit, total, total_pages, has_next, has_prev },
//     "links": [
//       {
//         "link_id":      String,
//         "link_url":     String,
//         "amount":       u64,
//         "mint":         String?,
//         "note":         String?,
//         "status":       String,
//         "expiry_at":    String?,
//         "claimer_wallet": String?,
//         "claimed_at":   String?,
//         "created_at":   String
//       }
//     ]
//   }
//
// ERRORS:
//   401 - unauthorized
//
// WORK TO DO:
//   [ ] Define MyLinksQuery dto    (page, limit, status)
//   [ ] Define LinkListItem dto    (fields above, excluding encrypted_escrow_secret)
//   [ ] Define MyLinksResponse dto (pagination + links vec)
//   [ ] In handler: extract JWT claims
//   [ ] In service: query `payment_links` WHERE creator_id = $1
//         optionally filter by status
//         ORDER BY created_at DESC, paginate
//         Run a background job or inline check: mark expired links
//           (status='active' AND expiry_at < NOW() => status='expired')
//   [ ] Return paginated result
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 3. GET /link/{link_token}   (PUBLIC — no auth)
// -----------------------------------------------------------------------------
// PURPOSE  : Look up a link by its token. Returns metadata so the claim page
//            can display amount, note, status, expiry etc. before claiming.
//            Does NOT expose escrow secret.
//
// AUTH     : None
//
// PATH PARAM:
//   - link_token : String
//
// RESPONSE (200):
//   {
//     "link_id":      String,
//     "amount":       u64,
//     "mint":         String?,
//     "symbol":       String,   -- resolved from mint (e.g. "SOL", "USDC")
//     "note":         String?,
//     "status":       String,   -- "active" | "claimed" | "cancelled" | "expired"
//     "expiry_at":    String?,
//     "created_at":   String
//   }
//
// ERRORS:
//   404 - link_token not found
//
// WORK TO DO:
//   [ ] Define LinkInfoResponse dto
//   [ ] In handler: no auth extraction needed
//   [ ] In service: SELECT from payment_links WHERE link_token = $1
//   [ ] Inline expiry check: if status='active' AND expiry_at < NOW() => return status='expired'
//         (optionally UPDATE the row to 'expired' at this point)
//   [ ] Resolve mint symbol (lookup from known mints list or coingecko/helius)
//   [ ] Return info (never expose escrow private key)
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 4. POST /link/{link_token}/claim  (PUBLIC — no auth)
// -----------------------------------------------------------------------------
// PURPOSE  : Claim the funds in the link. Transfers from escrow to the
//            provided claimer wallet. Anyone with the link can call this.
//
// AUTH     : None
//
// PATH PARAM:
//   - link_token : String
//
// REQUEST BODY (JSON):
//   {
//     "claimer_wallet": String  -- Solana public key of recipient
//   }
//
// RESPONSE (200):
//   {
//     "signature":      String,  -- Solana tx signature
//     "amount":         u64,
//     "mint":           String?,
//     "claimer_wallet": String,
//     "claimed_at":     String
//   }
//
// ERRORS:
//   400 - invalid claimer_wallet address
//   404 - link_token not found
//   409 - already claimed / cancelled / expired
//   500 - transfer failure
//
// WORK TO DO:
//   [ ] Define ClaimLinkRequest dto   (claimer_wallet: String)
//   [ ] Define ClaimLinkResponse dto  (signature, amount, mint, claimer_wallet, claimed_at)
//   [ ] In handler: no JWT, just parse path + body
//   [ ] In service:
//         a. SELECT link WHERE link_token = $1, lock row (SELECT FOR UPDATE)
//         b. Validate status = 'active', expiry not passed
//         c. Validate claimer_wallet is a valid Solana pubkey
//         d. Decrypt the escrow keypair secret
//         e. Build + sign + broadcast Solana transfer tx
//              FROM escrow_pubkey TO claimer_wallet for full {amount} of {mint}
//              Sign directly with the decrypted escrow Keypair (not MPC)
//         f. UPDATE payment_links SET status='claimed', claimer_wallet=.., claimed_at=NOW()
//         g. Insert into `transactions` table (type='receive', user_id=creator_id,
//              from_address=escrow_pubkey, to_address=claimer_wallet, ...)
//         h. Return signature + claim details
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 5. DELETE /link/{link_token}/cancel
// -----------------------------------------------------------------------------
// PURPOSE  : Creator cancels an active link and reclaims the escrowed funds
//            back to their own wallet.
//
// AUTH     : Required (JWT Bearer) — must be the creator
//
// PATH PARAM:
//   - link_token : String
//
// RESPONSE (200):
//   {
//     "signature": String,  -- Solana tx signature returning funds to creator
//     "status":    "cancelled"
//   }
//
// ERRORS:
//   401 - unauthorized
//   403 - not the creator of this link
//   404 - link_token not found
//   409 - link is not active (already claimed/cancelled/expired)
//
// WORK TO DO:
//   [ ] Define CancelLinkResponse dto (signature, status)
//   [ ] In handler: extract JWT claims (user_id)
//   [ ] In service:
//         a. SELECT link WHERE link_token = $1, lock row
//         b. Verify creator_id = jwt user_id (else 403)
//         c. Verify status = 'active' (else 409)
//         d. Decrypt escrow keypair secret
//         e. Build + sign + broadcast Solana transfer tx
//              FROM escrow_pubkey TO creator_wallet (creator's wallet_pubkey from users table)
//         f. UPDATE payment_links SET status='cancelled', updated_at=NOW()
//         g. Insert into `transactions` table (type='receive', amount=amount, ...)
//         h. Return signature + "cancelled" status
// -----------------------------------------------------------------------------

pub mod dto;
pub mod services;
pub mod routes;
pub mod handlers;
