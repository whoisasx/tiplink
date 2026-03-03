use actix_web::{middleware::from_fn, web};
use crate::middlewares::auth_middleware;
use super::services::*;

pub fn configure_links_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/link")
            // Public routes — no auth required; registered first so the
            // empty-prefix auth scope below does not shadow them.
            .service(handle_lookup_link)
            .service(handle_claim_link)
            // Protected routes — require a valid JWT
            .service(
                web::scope("")
                    .wrap(from_fn(auth_middleware))
                    .service(handle_create_link)
                    .service(handle_get_links)
                    .service(handle_cancel_link),
            ),
    );
}
