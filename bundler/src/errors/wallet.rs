use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::base::{DatabaseError, ErrorResponse, ProviderError};

#[derive(Debug, Display)]
pub enum WalletError {
    Database(String),
    Provider(String),
}

impl ResponseError for WalletError {
    fn status_code(&self) -> StatusCode {
        match self {
            WalletError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            WalletError::Provider(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            WalletError::Database(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
            WalletError::Provider(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
        }
    }
}

impl From<DatabaseError> for WalletError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => WalletError::Database(String::from("Record not found")),
            DatabaseError::ServerError(err) => WalletError::Database(err),
        }
    }
}

impl From<ProviderError> for WalletError {
    fn from(error: ProviderError) -> Self {
        WalletError::Provider(error.0)
    }
}
