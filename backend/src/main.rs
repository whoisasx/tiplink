use std::{io::Result};
use actix_web::{App, HttpResponse, HttpServer, Responder, delete, get, post, web};

mod routes;
use routes::*;

#[actix_web::main]
async fn main()-> Result<()>{

  HttpServer::new(move || {
    App::new()
    .configure(scoped_user)
  })
  .bind(("127.0.0.1", 3000))?
  .run()
  .await
}