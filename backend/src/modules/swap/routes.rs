use actix_web::{middleware::from_fn, web};
use crate::middlewares::auth_middleware;
use super::services::*;

pub fn configure_swap_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/swap")
            .wrap(from_fn(auth_middleware))
            .service(handle_get_swap_quote)
            .service(handle_execute_swap)
            .service(handle_get_swap_history),
    );
}
