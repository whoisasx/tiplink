use actix_web::{HttpResponse, http::StatusCode};

#[derive(serde::Serialize)]
pub struct Response<T>{
  pub success: bool,
  pub message: String,
  pub status_code: u16,
  pub value: Option<T>
}
impl<T> Response<T>{
  pub fn new(success:bool,message:String,status_code:u16,value:Option<T>) -> Response<T>{
    Response{
      success,
      message,
      status_code,
      value
    }
  }
}

pub fn send_response<T: serde::Serialize>(response: Response<T>) -> HttpResponse{
  match response.success{
    true=>{
      success_response(response)
    },
    false=>{
      failure_response(response)
    }
  }
}

pub fn success_response<T: serde::Serialize>(response: Response<T>) -> HttpResponse{
  HttpResponse::Ok().status(StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::OK)).json(response)
}
pub fn failure_response<T: serde::Serialize>(response: Response<T>) -> HttpResponse{
  HttpResponse::ExpectationFailed().status(StatusCode::from_u16(response.status_code).unwrap_or(StatusCode::EXPECTATION_FAILED)).json(response)
}