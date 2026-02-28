use actix_web::web;
use crate::handlers::*;
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(handle_create_wallet)
            .service(handle_transfer)
            .service(handle_sign_and_send),
    );
}
