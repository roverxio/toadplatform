use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::base::{DatabaseError, ErrorResponse, ProviderError};

#[derive(Debug, Display)]
pub enum AdminError {
    Unauthorized,
    InvalidCurrency,
    ValidationError(String),
    Database(String),
    Provider(String),
}

impl ResponseError for AdminError {
    fn status_code(&self) -> StatusCode {
        match self {
            AdminError::Unauthorized => StatusCode::UNAUTHORIZED,
            AdminError::InvalidCurrency => StatusCode::BAD_REQUEST,
            AdminError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AdminError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AdminError::Provider(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AdminError::Unauthorized => HttpResponse::Unauthorized()
                .json(ErrorResponse::from(String::from("Invalid credentials"))),
            AdminError::InvalidCurrency => HttpResponse::BadRequest()
                .json(ErrorResponse::from(String::from("Invalid currency"))),
            AdminError::ValidationError(error) => {
                HttpResponse::BadRequest().json(ErrorResponse::from(error.clone()))
            }
            AdminError::Database(error) => {
                error!("{error}");
                HttpResponse::InternalServerError().json(ErrorResponse::from(format!(
                    "Internal server error: {:?}",
                    error
                )))
            }
            AdminError::Provider(error) => {
                error!("{error}");
                HttpResponse::InternalServerError().json(ErrorResponse::from(format!(
                    "Internal server error: {:?}",
                    error
                )))
            }
        }
    }
}

impl From<DatabaseError> for AdminError {
    fn from(error: DatabaseError) -> Self {
        AdminError::Database(error.0)
    }
}

impl From<ProviderError> for AdminError {
    fn from(error: ProviderError) -> Self {
        AdminError::Provider(error.0)
    }
}
