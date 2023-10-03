use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;

use crate::errors::base::ErrorResponse;

#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalServer(String),
    Unauthorized,
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
