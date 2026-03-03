use actix_web::{middleware::from_fn, web};
use crate::middlewares::attach_claims_middleware;
use super::services::*;

pub fn configure_links_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/link")
            // Attach JWT claims when present; handlers decide whether auth is required.
            .wrap(from_fn(attach_claims_middleware))
            // ── Protected (handlers check extensions for JwtClaims) ──
            // Specific literal paths must be registered BEFORE wildcard /{link_token}.
            .service(handle_create_link)   // POST  /create
            .service(handle_get_links)     // GET   /my
            // ── Public ──
            // /{link_token}/claim has an extra segment, register before /{link_token}
            .service(handle_claim_link)    // POST  /{link_token}/claim
            // ── Protected wildcard (different method — DELETE — so won't shadow GET) ──
            .service(handle_cancel_link)   // DELETE /{link_token}/cancel
            // ── Public generic wildcard (last) ──
            .service(handle_lookup_link)   // GET   /{link_token}
    );
}
