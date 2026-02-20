use actix_web::{HttpResponse, Responder, delete, get, web};
use crate::{config::Config, modules::auth::{dto::{CallbackQuery, GoogleTokenResponse, GoogleUserResponse}, services::save_or_create_user}, utils::{Response, send_response}};


#[get("/google/init")]
pub async fn google_init(config:web::Data<Config>)->impl Responder{
  let google_client_id=&config.google_oauth_client_id;
  let random_string=&config.random_string;
  let redirect_uri=&config.redirect_url;

  let url = format!(
    "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&state={}",
    google_client_id, redirect_uri, random_string
  );

  send_response(Response::new(true, String::from("redirect to google api"), 200, Some(url)))
}

#[get("/callback/google")]
pub async fn google_callback(query:web::Query<CallbackQuery>,config:web::Data<Config>)-> actix_web::Result<impl Responder>{
  if query.state!=config.random_string{
    return Ok(send_response(Response::new(false,String::from("Invalid state"),404,None::<String>)));
  }
  
  let client = reqwest::Client::new();
  let token_info = client.post("https://oauth2.googleapis.com/token")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(format!(
      "code={}&client_id={}&client_secret={}&redirect_uri={}&grant_type=authorization_code",
      query.code,
      config.google_oauth_client_id,
      config.google_oauth_client_secret,
      config.redirect_url
    ))
    .send()
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .json::<GoogleTokenResponse>()
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

  let user_info = client.get("https://www.googleapis.com/oauth2/v2/userinfo")
    .header("Authorization", format!("Bearer {}", token_info.access_token))
    .send()
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    .json::<GoogleUserResponse>()
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

  if !save_or_create_user(token_info, user_info).await {
    return Ok(send_response(Response::new(false, String::from("Internal server error"), 500, None::<String>)));
  }


  Ok(HttpResponse::Ok().body("sign in end point"))
}

#[get("/refresh")]
pub async fn refresh_token()->impl Responder{
  HttpResponse::Ok().body("get user end point")
}

#[delete("/logout")]
pub async fn log_out()->impl Responder{
  HttpResponse::Ok().body("log out end point")
}

pub fn scoped_auth(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("auth")
      .service(google_init)
      .service(google_callback)
      .service(refresh_token)
      .service(log_out)
  );
}