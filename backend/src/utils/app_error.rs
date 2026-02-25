use actix_web::{ HttpResponse, ResponseError, http::StatusCode};
use derive_more::Display;

use super::api_response::*;

#[derive(Debug, Display)]
pub enum AppError {
    #[display("Bad request: {_0}")]
    BadRequest(String),

    #[display("Unauthorized")]
    Unauthorized,

    #[display("Forbidden")]
    Forbidden,

    #[display("Not found: {_0}")]
    NotFound(String),

    #[display("Conflict: {_0}")]
    Conflict(String),

    #[display("Internal server error")]
    Internal,
}

impl ResponseError for AppError{
  fn status_code(&self) -> StatusCode {
    match self {
      AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
      AppError::Unauthorized => StatusCode::UNAUTHORIZED,
      AppError::Forbidden => StatusCode::FORBIDDEN,
      AppError::NotFound(_) => StatusCode::NOT_FOUND,
      AppError::Conflict(_) => StatusCode::CONFLICT,
      AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR
    }
  }

  fn error_response(&self) -> HttpResponse{
    ApiResponse::<()>::error(&self.to_string(),self.status_code())
  }
}