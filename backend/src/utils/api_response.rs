use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T>{
  pub success:bool,
  pub message:String,
  pub result: Option<T>,
  pub error: Option<String>
}

impl<T:Serialize> ApiResponse<T>{
  pub fn success(message: &str, data: T, status: StatusCode) -> HttpResponse{
    HttpResponse::build(status).json(ApiResponse {
      success: true,
      message: message.to_string(),
      result: Some(data),
      error: None
    })
  }

  pub fn ok(message: &str, data: T) -> HttpResponse{
    Self::success(message, data, StatusCode::OK)
  }
  pub fn created(message: &str, data: T) -> HttpResponse{
    Self::success(message, data, StatusCode::CREATED)
  }
  pub fn accepted(message: &str, data: T) -> HttpResponse{
    Self::success(message, data, StatusCode::ACCEPTED)
  }
  pub fn no_content(message: &str) -> HttpResponse {
    // Self::success(message, None, StatusCode::NO_CONTENT)
    HttpResponse::NoContent().json(ApiResponse::<()>{
      success: true,
      message: message.to_string(),
      result: None,
      error: None
    })
  }

  pub fn error(message: &str, status: StatusCode) -> HttpResponse{
    HttpResponse::build(status).json(ApiResponse::<()>{
      success: false,
      message: message.to_string(),
      result: None,
      error: Some(message.to_string())
    })
  }
}