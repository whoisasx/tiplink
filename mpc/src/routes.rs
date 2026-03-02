use actix_web::{middleware::from_fn, web};
use crate::handlers::*;
use crate::middlewares::mpc_auth_middleware;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(from_fn(mpc_auth_middleware))
            .service(handle_create_wallet)
            .service(handle_transfer)
            .service(handle_sign_and_send),
    );
}
