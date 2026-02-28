use actix_web::{middleware::from_fn, web};

use crate::middlewares::auth_middleware;
use super::services::*;

pub fn configure_user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("user")
            .wrap(from_fn(auth_middleware))
            .service(handle_get_current_user),
    );
}