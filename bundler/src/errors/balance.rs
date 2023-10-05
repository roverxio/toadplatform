use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::base::{DatabaseError, ErrorResponse, ProviderError};

#[derive(Debug, Display)]
pub enum BalanceError {
    NotFound,
    Database(String),
    Provider(String),
    InvalidCurrency,
}

impl ResponseError for BalanceError {
    fn status_code(&self) -> StatusCode {
        match self {
            BalanceError::NotFound => StatusCode::NOT_FOUND,
            BalanceError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BalanceError::Provider(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BalanceError::InvalidCurrency => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            BalanceError::NotFound => {
                HttpResponse::NotFound().json(ErrorResponse::from(String::from("User not found")))
            }
            BalanceError::Database(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
            BalanceError::Provider(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
            BalanceError::InvalidCurrency => HttpResponse::BadRequest()
                .json(ErrorResponse::from(String::from("Invalid chain/currency"))),
        }
    }
}

impl From<DatabaseError> for BalanceError {
    fn from(error: DatabaseError) -> Self {
        BalanceError::Database(error.0)
    }
}

impl From<ProviderError> for BalanceError {
    fn from(error: ProviderError) -> Self {
        BalanceError::Provider(error.0)
    }
}
