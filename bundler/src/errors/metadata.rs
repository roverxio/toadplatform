use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::base::{DatabaseError, ErrorResponse};

#[derive(Debug, Display)]
pub enum MetadataError {
    Database(String),
}

impl ResponseError for MetadataError {
    fn status_code(&self) -> StatusCode {
        match self {
            MetadataError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            MetadataError::Database(error) => {
                error!("{error}");
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::from(String::from("Internal server error")))
            }
        }
    }
}

impl From<DatabaseError> for MetadataError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => MetadataError::Database(String::from("Record not found")),
            DatabaseError::ServerError(err) => MetadataError::Database(err),
        }
    }
}
