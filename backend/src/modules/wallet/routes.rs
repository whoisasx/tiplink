use actix_web::{HttpResponse, Responder, get, post, web};

#[get("/balance")]
pub async fn get_balanace()->impl Responder{
  HttpResponse::Ok().body("hello from solana")
}

#[get("/transactions")]
pub async fn get_txns()->impl Responder{
  HttpResponse::Ok().body("get the txns")
}

#[post("/send")]
pub async fn send_balance()->impl Responder{
  HttpResponse::Ok().body("send solana")
}

#[get("/send/estimate")]
pub async fn send_balance_estimate()->impl Responder{
  HttpResponse::Ok().body("send balance estimate")
}

pub fn scoped_wallet(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("/wallet")
    .service(get_balanace)
    .service(get_txns)
    .service(send_balance)
    .service(send_balance_estimate)
  );
}