use actix_web::{middleware::from_fn, web};
use crate::middlewares::auth_middleware;
use super::services::*;

pub fn configure_links_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/link")
            .service(
                web::scope("")
                    .wrap(from_fn(auth_middleware))
                    .service(handle_create_link)
                    .service(handle_get_links)
                    .service(handle_cancel_link),
            )
            .service(handle_lookup_link)
            .service(handle_claim_link),
    );
}
