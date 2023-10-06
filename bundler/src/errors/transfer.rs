use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::{DatabaseError, ErrorResponse, ProviderError};

#[derive(Debug, Display)]
pub enum TransferError {
    NotFound,
    Provider(String),
    Database(String),
}

impl ResponseError for TransferError {
    fn status_code(&self) -> StatusCode {
        match self {
            TransferError::NotFound => StatusCode::NOT_FOUND,
            TransferError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            TransferError::Provider(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            TransferError::NotFound => {
                HttpResponse::NotFound().json(ErrorResponse::from(String::from("User not found")))
            }
            TransferError::Database(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
            TransferError::Provider(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
        }
    }
}

impl From<DatabaseError> for TransferError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => TransferError::Database(String::from("Record not found")),
            DatabaseError::ServerError(err) => TransferError::Database(err),
        }
    }
}

impl From<ProviderError> for TransferError {
    fn from(error: ProviderError) -> Self {
        TransferError::Provider(error.0)
    }
}
