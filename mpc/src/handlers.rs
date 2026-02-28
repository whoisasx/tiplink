use actix_web::{post, web::{Data, Json}, HttpResponse};

use crate::{
    config::Config,
    dto::*,
    errors::MpcError,
    services,
};

fn ok<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

#[post("/wallet/create")]
pub async fn handle_create_wallet(
    body:   Json<CreateWalletRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, MpcError> {
    let req = body.into_inner();

    tracing::info!(user_id = %req.user_id, "wallet/create request");

    let pubkey = services::create_wallet(&req.user_id, &config).await?;

    Ok(ok(CreateWalletResponse { pubkey }))
}

#[post("/transfer")]
pub async fn handle_transfer(
    body:   Json<TransferRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, MpcError> {
    let req = body.into_inner();

    tracing::info!(
        from    = %req.from,
        to      = %req.to,
        amount  = req.amount,
        "transfer request"
    );

    let signature = services::execute_transfer(req, &config).await?;

    Ok(ok(TransferResponse { signature }))
}

#[post("/sign-and-send")]
pub async fn handle_sign_and_send(
    body:   Json<SignAndSendRequest>,
    config: Data<Config>,
) -> Result<HttpResponse, MpcError> {
    let req = body.into_inner();

    tracing::info!(user_id = %req.user_id, wallet = %req.wallet_pubkey, "sign-and-send request");

    let signature = services::sign_and_send(req, &config).await?;

    Ok(ok(SignAndSendResponse { signature }))
}
