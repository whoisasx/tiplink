use actix_web::{middleware::from_fn, web};
use crate::middlewares::auth_middleware;
use super::services::*;

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("auth")
            .service(initiate_google_auth)
            .service(handle_google_callback)
            .service(handle_token_refresh)
            .service(
                web::scope("")
                    .wrap(from_fn(auth_middleware))
                    .service(handle_logout),
            ),
    );
}