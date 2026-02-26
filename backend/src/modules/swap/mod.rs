// =============================================================================
// MODULE: swap
// PURPOSE: Jupiter-powered token swaps (SOL <-> SPL, SPL <-> SPL)
//          Uses Jupiter Aggregator API for quotes & swap transactions.
//          All routes require JWT auth (auth_middleware).
// =============================================================================

// -----------------------------------------------------------------------------
// DB TABLES NEEDED
// -----------------------------------------------------------------------------
// Uses existing:
//   transactions  (id, user_id, signature, type='swap', status, amount, mint,
//                  from_address, to_address, fee, metadata, confirmed_at,
//                  created_at, updated_at)
//
// metadata JSONB for swap will contain:
//   {
//     "input_mint":    "So11....",
//     "output_mint":   "EPjF....",
//     "input_amount":  1000000,       -- in base units
//     "output_amount": 9870000,       -- in base units (after slippage)
//     "slippage_bps":  50,
//     "price_impact":  "0.12",        -- percent string
//     "route_label":   "Raydium",     -- best route label from Jupiter
//     "quote_id":      "abc123"        -- Jupiter quoteResponse id (optional)
//   }
// -----------------------------------------------------------------------------

// =============================================================================
// ROUTE MAP
// =============================================================================

// -----------------------------------------------------------------------------
// 1. GET /swap/quote
// -----------------------------------------------------------------------------
// PURPOSE  : Fetch a swap quote from Jupiter for a given input/output pair.
//
// AUTH     : Required (JWT Bearer)
//
// QUERY PARAMS:
//   - input_mint   : String  -- mint address of token to swap FROM (or "SOL")
//   - output_mint  : String  -- mint address of token to swap TO   (or "SOL")
//   - amount       : u64     -- input amount in base units (lamports / decimals)
//   - slippage_bps : u16     -- slippage tolerance in basis points (e.g. 50 = 0.5%)
//
// RESPONSE (200):
//   {
//     "input_mint":    String,
//     "output_mint":   String,
//     "input_amount":  u64,
//     "output_amount": u64,    -- estimated out after slippage
//     "price_impact":  String, -- e.g. "0.12%"
//     "route_label":   String, -- e.g. "Raydium"
//     "slippage_bps":  u16,
//     "quote_raw":     Object  -- full Jupiter quoteResponse (pass-through for FE)
//   }
//
// ERRORS:
//   400 - missing/invalid params
//   502 - Jupiter API error / unreachable
//
// WORK TO DO:
//   [ ] Define SwapQuoteQuery dto (input_mint, output_mint, amount, slippage_bps)
//   [ ] Define SwapQuoteResponse dto
//   [ ] In handler: validate query params (amount > 0, mints not empty/equal)
//   [ ] In service: call Jupiter v6 quote API
//         GET https://quote-api.jup.ag/v6/quote?inputMint=..&outputMint=..&amount=..&slippageBps=..
//   [ ] Deserialize Jupiter quoteResponse, map to SwapQuoteResponse
//   [ ] Return quote to caller (also cache quote_id in response for use in execute)
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 2. POST /swap/execute
// -----------------------------------------------------------------------------
// PURPOSE  : Execute the swap using a previously fetched Jupiter quote.
//            Backend fetches the swap transaction from Jupiter, reconstructs
//            + partially signs with the MPC wallet share, returns serialized
//            tx OR broadcasts directly depending on architecture choice.
//
// AUTH     : Required (JWT Bearer)
//
// REQUEST BODY (JSON):
//   {
//     "input_mint":    String,
//     "output_mint":   String,
//     "input_amount":  u64,
//     "output_amount": u64,    -- expected, used for slippage check
//     "slippage_bps":  u16,
//     "quote_raw":     Object  -- the full Jupiter quoteResponse from /quote
//   }
//
// RESPONSE (200):
//   {
//     "txn_id":    String,  -- internal transaction record UUID
//     "signature": String,  -- Solana tx signature (if broadcasted)
//     "status":    String   -- "pending" | "confirmed"
//   }
//
// ERRORS:
//   400 - invalid body / slippage exceeded
//   401 - unauthorized
//   502 - Jupiter swap-transaction API error
//   500 - MPC signing error / broadcast error
//
// WORK TO DO:
//   [ ] Define SwapExecuteRequest dto (input_mint, output_mint, amounts, slippage_bps, quote_raw)
//   [ ] Define SwapExecuteResponse dto (txn_id, signature, status)
//   [ ] In handler: extract JWT claims (user_id, wallet pubkey)
//   [ ] In service:
//         a. Call Jupiter v6 swap API to get the serialized transaction
//              POST https://quote-api.jup.ag/v6/swap  { quoteResponse, userPublicKey }
//         b. Deserialize the returned base64 transaction
//         c. Reconstruct the VersionedTransaction, apply MPC partial sign
//              (use existing MPC/wallet-key infrastructure from wallet module)
//         d. Broadcast transaction to Solana RPC
//         e. Insert a 'pending' record into `transactions` table
//              (type='swap', mint=input_mint, amount=input_amount, metadata=swap details)
//         f. Return txn_id + signature + "pending" status
//   [ ] Optionally: kick off a background task / webhook to confirm the tx
//         and update transactions.status to 'confirmed' / 'failed'
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 3. GET /swap/history
// -----------------------------------------------------------------------------
// PURPOSE  : Return paginated list of past swaps for the authenticated user.
//            Delegates to the common transactions table filtered by type='swap'.
//
// AUTH     : Required (JWT Bearer)
//
// QUERY PARAMS:
//   - page  : u32  (default 1)
//   - limit : u32  (default 20, max 100)
//
// RESPONSE (200):
//   {
//     "pagination": { page, limit, total, total_pages, has_next, has_prev },
//     "swaps": [
//       {
//         "id":           String,   -- transaction UUID
//         "signature":    String,
//         "status":       String,   -- "pending" | "confirmed" | "failed"
//         "input_mint":   String,
//         "output_mint":  String,
//         "input_amount": u64,
//         "output_amount":u64,
//         "price_impact": String,
//         "route_label":  String,
//         "created_at":   DateTime
//       }
//     ]
//   }
//
// ERRORS:
//   401 - unauthorized
//
// WORK TO DO:
//   [ ] Define SwapHistoryQuery dto (page, limit) - can reuse/extend TransactionFilterQuery
//   [ ] Define SwapHistoryItem + SwapHistoryResponse dtos
//   [ ] In handler: extract JWT claims
//   [ ] In service: query `transactions` WHERE user_id = $1 AND type = 'swap'
//         ORDER BY created_at DESC, with LIMIT/OFFSET for pagination
//   [ ] Flatten metadata JSONB fields into the SwapHistoryItem struct
//   [ ] Return paginated result
// -----------------------------------------------------------------------------

pub mod dto;
pub mod services;
pub mod routes;
pub mod handlers;