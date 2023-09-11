use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalServer(String),
    Unauthorized,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Error {
    #[serde(skip_serializing_if = "String::is_empty")]
    message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub data: Value,
    pub err: Error,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::BadRequest(error) => {
                HttpResponse::BadRequest().json(ErrorResponse::from(String::from(error)))
            }
            ApiError::NotFound(message) => {
                HttpResponse::NotFound().json(ErrorResponse::from(String::from(message)))
            }
            ApiError::InternalServer(message) => {
                HttpResponse::InternalServerError().json(ErrorResponse::from(String::from(message)))
            }
            ApiError::Unauthorized => HttpResponse::Unauthorized()
                .json(ErrorResponse::from(String::from("Unauthorized request"))),
        }
    }
}

impl From<String> for ErrorResponse {
    fn from(error: String) -> Self {
        ErrorResponse {
            data: json!({}),
            err: Error { message: error },
        }
    }
}
