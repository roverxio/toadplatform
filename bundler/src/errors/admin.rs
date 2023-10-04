use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use log::error;

use crate::errors::base::{DatabaseError, ErrorResponse};

#[derive(Debug, Display)]
pub enum AdminError {
    Unauthorized,
    Database(String),
}

impl ResponseError for AdminError {
    fn status_code(&self) -> StatusCode {
        match self {
            AdminError::Unauthorized => StatusCode::UNAUTHORIZED,
            AdminError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AdminError::Unauthorized => HttpResponse::Unauthorized()
                .json(ErrorResponse::from(String::from("Invalid credentials"))),
            AdminError::Database(error) => {
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
