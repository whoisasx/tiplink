use actix_web::{HttpRequest, HttpResponse, Responder, cookie::Cookie, delete, get, middleware::from_fn, web};
use chrono::Utc;
use crate::{config::Config, middlewares::auth_middleware, modules::{TokenDesign, auth::{dto::{CallbackQuery, GoogleTokenResponse, GoogleUserResponse}, services::{create_jwt_token, get_refresh_token_details, get_user_details, hash_token, upsert_refresh_token, upsert_user, upsert_wallet}}}, utils::{Response, send_response}};
use serde_json::json;


#[get("/google/init")]
pub async fn google_init(config:web::Data<Config>)->impl Responder{
  let google_client_id=&config.google_oauth_client_id;
  let random_string=&config.random_string;
  let redirect_uri=&config.redirect_url;

  let url = format!(
    "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&access_type=offline&prompt=consent&scope=openid%20email%20profile&state={}",
    google_client_id, redirect_uri, random_string
  );

  send_response(Response::new(true, String::from("redirect to google api"), 200, Some(url)))
}

#[get("/callback/google")]
pub async fn google_callback(query:web::Query<CallbackQuery>,config:web::Data<Config>)-> actix_web::Result<impl Responder>{
  if query.state!=config.random_string{
    return Ok(HttpResponse::Found()
      .append_header(("Location", config.client_origin.clone()))
      .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
      .cookie(Cookie::build("auth_message", "state_not_found").path("/").http_only(false).finish())
      .finish())
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

  if !upsert_user(&token_info, &user_info).await {
    return Ok(HttpResponse::Found()
      .append_header(("Location", config.client_origin.clone()))
      .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
      .cookie(Cookie::build("auth_message", "failed_to_save_user").path("/").http_only(false).finish())
      .finish())
  }

  if !upsert_refresh_token(&token_info,&user_info).await {
    return Ok(HttpResponse::Found()
      .append_header(("Location", config.client_origin.clone()))
      .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
      .cookie(Cookie::build("auth_message", "failed_to_save_refresh_token").path("/").http_only(false).finish())
      .finish())
  }

  let wallet=upsert_wallet(&token_info, &user_info).await;
  if wallet.len()<32 || wallet.len()>44 {
    return Ok(HttpResponse::Found()
      .append_header(("Location", config.client_origin.clone()))
      .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
      .cookie(Cookie::build("auth_message", "failed_to_get_wallet_address").path("/").http_only(false).finish())
      .finish())
  }

  //TODO: add .secure(true) for production -> @87 line
  Ok(HttpResponse::Found()
      .append_header(("Location", config.client_origin.clone()))
      .cookie(Cookie::build("auth_status", "success").path("/").http_only(false).finish())
      .cookie(Cookie::build("refresh_token", token_info.refresh_token).path("/").http_only(true).finish())
      .finish()
    )
}

#[get("/refresh")]
pub async fn refresh_token(req:HttpRequest)->impl Responder{
  let refresh_token=match req.cookie("refresh_token"){
    Some(r)=> r.value().to_string(),
    None=>{
      return send_response(Response::new(false,String::from("unauthorised, refresh token missing."),401,None::<String>))
    }
  };

  let hashed_token=hash_token(&refresh_token);
  let token_record= match get_refresh_token_details(&hashed_token).await{
    Some(h)=>h,
    None=>{
      return HttpResponse::Unauthorized()
        .cookie(Cookie::build("refresh_token", "").path("/").max_age(actix_web::cookie::time::Duration::seconds(0)).finish())
        .json(Response::new(false,String::from("unauthorised, refresh token invalid."),401,None::<String>));
    }
  };

  if token_record.expires_at<Utc::now().timestamp() || token_record.token_hash !=hashed_token {
    return HttpResponse::Unauthorized()
      .cookie(Cookie::build("refresh_token", "").path("/").max_age(actix_web::cookie::time::Duration::seconds(0)).finish())
      .json(Response::new(false,String::from("unauthorised, refresh token expired/invalid."),401,None::<String>));
  }

  let user_record= match get_user_details(&token_record.user_id).await{
    Some(u)=>u,
    None => {
      return HttpResponse::Unauthorized()
        .cookie(Cookie::build("refresh_token", "").path("/").max_age(actix_web::cookie::time::Duration::seconds(0)).finish())
        .json(Response::new(false,String::from("unauthorised, user not found."),401,None::<String>));
    }
  };

  let jwt_token=create_jwt_token(TokenDesign::new(user_record.google_sub, user_record.email, user_record.wallet));

  send_response(Response::new(true, String::from("jwt-token refreshed"), 200, Some(json!({"jwt_token": jwt_token}))))

}

#[delete("/logout")]
pub async fn log_out()->impl Responder{
  HttpResponse::Ok()
    .cookie(Cookie::build("refresh_token", "").path("/").max_age(actix_web::cookie::time::Duration::seconds(0)).finish())
    .json(json!({"status":"user logged out."}))
}

pub fn scoped_auth(cfg: &mut web::ServiceConfig){
  cfg.service(
    web::scope("auth")
      .service(google_init)
      .service(google_callback)
      .service(refresh_token)
      .service(
        web::scope("")
          .wrap(from_fn(auth_middleware))
          .service(log_out)
      )
  );
}