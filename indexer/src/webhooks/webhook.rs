use actix_web::{Error, HttpRequest, HttpResponse, post, web::Json};
use super::dto::*;

#[post("/webhook")]
pub async fn handle_hooks(
    _req: HttpRequest,
    payload: Json<HeliusWebhookPayload>,
) -> Result<HttpResponse, Error> {
    let txns = payload.into_inner();

    for txn in &txns {
        tracing::info!(
            signature  = %txn.signature,
            txn_type   = ?txn.txn_type,
            source     = ?txn.source,
            slot       = ?txn.slot,
            timestamp  = ?txn.timestamp,
            "received helius webhook"
        );
    }

    Ok(HttpResponse::Ok().finish())
}
