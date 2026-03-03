use actix_web::{Error, HttpRequest, HttpResponse, cookie::Cookie, delete, get, web};
use chrono::{Duration, Utc};
use uuid::Uuid;
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

    if !upsert_wallet(
        &token_info,
        &user_info,
        &config.mpc_server_url,
        &config.mpc_secret,
        config.helius_api_key.as_deref(),
        config.helius_webhook_id.as_deref(),
    ).await {
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
pub async fn handle_token_refresh(req: HttpRequest, config: web::Data<Config>) -> Result<HttpResponse, AppError> {
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

    let wallet = match user_record.wallet {
        Some(w) => w,
        // Wallet isn't ready yet (MPC provisioning lag). Return 503 so the
        // client can retry, but do NOT clear the refresh_token cookie — the
        // token is still valid and should not be invalidated.
        None => return Ok(HttpResponse::ServiceUnavailable()
            .json(ApiResponse::<String> {
                success: false,
                message: "Wallet not ready yet.".to_string(),
                result: None,
                error: Some("Wallet not ready yet.".to_string()),
            })),
    };

    let jwt_token = create_jwt_token(
        JwtClaims::new(user_record.id.clone(), user_record.email, wallet),
        &config,
    ).map_err(|e| {
        tracing::error!("JWT encoding failed: {e}");
        AppError::Internal
    })?;

    const ROTATE_THRESHOLD_SECS: i64 = 7 * 24 * 3600;
    let secs_remaining = token_record.expires_at - Utc::now().timestamp();
    let outgoing_cookie_token = if secs_remaining <= ROTATE_THRESHOLD_SECS {
        let _ = store::users::revoke_refresh_token(&hashed_token).await;
        let new_raw_token  = Uuid::new_v4().to_string();
        let new_token_hash = hash_refresh_token(&new_raw_token);
        let new_expires_at = Utc::now() + Duration::days(180);
        if let Ok(user_uuid) = Uuid::parse_str(&user_record.id) {
            let _ = store::users::insert_refresh_token(
                user_uuid, &new_token_hash, new_expires_at, None, None,
            ).await;
        }
        new_raw_token
    } else {
        refresh_token.clone()
    };

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refresh_token", outgoing_cookie_token)
                .path("/")
                .http_only(true)
                .finish(),
        )
        .json(json!({
            "success": true,
            "message": "JWT token refreshed",
            "result": { "jwt_token": jwt_token },
            "error": null
        })))
}

#[delete("/logout")]
pub async fn handle_logout(req: HttpRequest) -> Result<HttpResponse, AppError> {
    if let Some(cookie) = req.cookie("refresh_token") {
        let hashed = hash_refresh_token(cookie.value());
        if let Err(e) = store::users::revoke_refresh_token(&hashed).await {
            tracing::warn!("Failed to revoke refresh token on logout: {e}");
        }
    }

    Ok(HttpResponse::Ok()
        .cookie(
            Cookie::build("refresh_token", "")
                .path("/")
                .http_only(true)
                .max_age(actix_web::cookie::time::Duration::seconds(0))
                .finish(),
        )
        .json(json!({ "success": true, "message": "User logged out." })))
}
