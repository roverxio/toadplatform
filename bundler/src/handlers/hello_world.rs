use actix_web::{Error, HttpResponse};

use crate::models::response::BaseResponse;
use crate::services::HelloWorldService;

pub async fn hello_world() -> Result<HttpResponse, Error> {
    let hello = HelloWorldService::hello_world()?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(hello)))
}
