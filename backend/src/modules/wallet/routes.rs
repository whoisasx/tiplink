use actix_web::{middleware::from_fn, web};

use crate::middlewares::auth_middleware;

use super::services::{
    handle_estimate_fee, handle_get_all_balances, handle_get_token_balance,
    handle_get_transaction, handle_get_transactions, handle_send_transaction,
};

pub fn configure_wallet_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/wallet")
            .wrap(from_fn(auth_middleware))
            .service(handle_get_all_balances)
            .service(handle_get_token_balance)
            .service(handle_get_transactions)
            .service(handle_get_transaction)
            .service(handle_send_transaction)
            .service(handle_estimate_fee),
    );
}