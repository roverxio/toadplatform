use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::{DatabaseError, ErrorResponse};

#[derive(Debug, Display)]
pub enum TransactionError {
    NotFound,
    Database(String),
}

impl ResponseError for TransactionError {
    fn status_code(&self) -> StatusCode {
        match self {
            TransactionError::NotFound => StatusCode::NOT_FOUND,
            TransactionError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            TransactionError::NotFound => HttpResponse::NotFound()
                .json(ErrorResponse::from(String::from("Transaction not found"))),
            TransactionError::Database(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
        }
    }
}

impl From<DatabaseError> for TransactionError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => TransactionError::NotFound,
            DatabaseError::ServerError(err) => TransactionError::Database(err),
        }
    }
}
