use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use derive_more::Display;
use serde_json::json;

#[derive(Debug, Display)]
pub enum MpcError {
    #[display("Bad request: {_0}")]
    BadRequest(String),

    #[display("Unauthorized")]
    Unauthorized,

    #[display("Key not found for user: {_0}")]
    KeyNotFound(String),

    #[display("Signing failed: {_0}")]
    SigningFailed(String),

    #[display("Broadcast failed: {_0}")]
    BroadcastFailed(String),

    #[display("Internal error")]
    Internal,
}

impl ResponseError for MpcError {
    fn status_code(&self) -> StatusCode {
        match self {
            MpcError::BadRequest(_)      => StatusCode::BAD_REQUEST,
            MpcError::Unauthorized       => StatusCode::UNAUTHORIZED,
            MpcError::KeyNotFound(_)     => StatusCode::NOT_FOUND,
            MpcError::SigningFailed(_)   => StatusCode::UNPROCESSABLE_ENTITY,
            MpcError::BroadcastFailed(_) => StatusCode::BAD_GATEWAY,
            MpcError::Internal           => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(json!({
            "success": false,
            "message": self.to_string(),
        }))
    }
}
