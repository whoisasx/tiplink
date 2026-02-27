use actix_web::{Error, HttpRequest, HttpResponse, cookie::Cookie, delete, get, web};
use chrono::Utc;
use crate::{
    config::Config,
    modules::JwtClaims,
    utils::{ApiResponse, AppError},
};
use serde_json::json;

use super::{handlers::*, dto::*};

#[get("/google/init")]
pub async fn initiate_google_auth(config: web::Data<Config>) -> Result<HttpResponse, AppError> {
    let google_client_id = &config.google_oauth_client_id;
    let csrf_state_token = &config.csrf_state_token;
    let redirect_uri = &config.redirect_url;

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&access_type=offline&prompt=consent&scope=openid%20email%20profile&state={}",
        google_client_id, redirect_uri, csrf_state_token
    );

    Ok(ApiResponse::ok("Google redirect URL", url))
}

#[get("/callback/google")]
pub async fn handle_google_callback(
    query: web::Query<OAuthCallbackQuery>,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    if query.state != config.csrf_state_token {
        return Ok(HttpResponse::Found()
            .append_header(("Location", config.client_origin.clone()))
            .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
            .cookie(Cookie::build("auth_message", "state_not_found").path("/").http_only(false).finish())
            .finish());
    }

    let client = reqwest::Client::new();

    let token_info = client
        .post("https://oauth2.googleapis.com/token")
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
        .json::<GoogleOAuthTokenResponse>()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let user_info = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", token_info.access_token))
        .send()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .json::<GoogleUserInfo>()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if !upsert_user(&token_info, &user_info).await {
        return Ok(HttpResponse::Found()
            .append_header(("Location", config.client_origin.clone()))
            .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
            .cookie(Cookie::build("auth_message", "failed_to_save_user").path("/").http_only(false).finish())
            .finish());
    }

    if !upsert_refresh_token(&token_info, &user_info).await {
        return Ok(HttpResponse::Found()
            .append_header(("Location", config.client_origin.clone()))
            .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
            .cookie(Cookie::build("auth_message", "failed_to_save_refresh_token").path("/").http_only(false).finish())
            .finish());
    }

    if !upsert_wallet(&token_info, &user_info, &config.mpc_server_url).await {
        return Ok(HttpResponse::Found()
            .append_header(("Location", config.client_origin.clone()))
            .cookie(Cookie::build("auth_status", "error").path("/").http_only(false).finish())
            .cookie(Cookie::build("auth_message", "failed_to_save_wallet").path("/").http_only(false).finish())
            .finish());
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", config.client_origin.clone()))
        .cookie(Cookie::build("auth_status", "success").path("/").http_only(false).finish())
        .cookie(Cookie::build("refresh_token", token_info.refresh_token).path("/").http_only(true).finish())
        .finish())
}

#[get("/refresh")]
pub async fn handle_token_refresh(req: HttpRequest) -> Result<HttpResponse, AppError> {
    let refresh_token = match req.cookie("refresh_token") {
        Some(r) => r.value().to_string(),
        None => return Err(AppError::Unauthorized("Refresh token not provided".to_string())),
    };

    let hashed_token = hash_refresh_token(&refresh_token);

    let token_record = match get_refresh_token_record(&hashed_token).await {
        Some(h) => h,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .cookie(
                    Cookie::build("refresh_token", "")
                        .path("/")
                        .max_age(actix_web::cookie::time::Duration::seconds(0))
                        .finish(),
                )
                .json(ApiResponse::<String> {
                    success: false,
                    message: "Unauthorized: invalid refresh token.".to_string(),
                    result: None,
                    error: Some("Unauthorized: invalid refresh token.".to_string()),
                }));
        }
    };

    if token_record.expires_at < Utc::now().timestamp() || token_record.token_hash != hashed_token {
        return Ok(HttpResponse::Unauthorized()
            .cookie(
                Cookie::build("refresh_token", "")
                    .path("/")
                    .max_age(actix_web::cookie::time::Duration::seconds(0))
                    .finish(),
            )
            .json(ApiResponse::<String> {
                success: false,
                message: "Unauthorized: refresh token expired.".to_string(),
                result: None,
                error: Some("Unauthorized: refresh token expired.".to_string()),
            }));
    }

    let user_record = match get_user_record(&token_record.user_id).await {
        Some(u) => u,
        None => {
            return Ok(HttpResponse::Unauthorized()
                .cookie(
                    Cookie::build("refresh_token", "")
                        .path("/")
                        .max_age(actix_web::cookie::time::Duration::seconds(0))
                        .finish(),
                )
                .json(ApiResponse::<String> {
                    success: false,
                    message: "Unauthorized: user not found.".to_string(),
                    result: None,
                    error: Some("Unauthorized: user not found.".to_string()),
                }));
        }
    };

    let jwt_token = create_jwt_token(JwtClaims::new(
        user_record.id,
        user_record.email,
        match user_record.wallet {
            Some(w) => w,
            None => return Ok(HttpResponse::ServiceUnavailable()
                .cookie(
                    Cookie::build("refresh_token", "")
                        .path("/")
                        .max_age(actix_web::cookie::time::Duration::seconds(0))
                        .finish(),
                )
                .json(ApiResponse::<String> {
                    success: false,
                    message: "Wallet not ready yet.".to_string(),
                    result: None,
                    error: Some("Wallet not ready yet.".to_string()),
                })),
        },
    ));

    Ok(ApiResponse::ok("JWT token refreshed", json!({ "jwt_token": jwt_token })))
}

#[delete("/logout")]
pub async fn handle_logout() -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refresh_token", "")
                .path("/")
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .finish(),
        )
        .json(json!({ "status": "user logged out." })))
}
