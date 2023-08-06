use actix_web::http::header::HeaderName;
use actix_web::HttpRequest;
use actix_web::web::Json;
use serde::Serialize;

use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;

pub fn respond_json<T>(data: T) -> Result<Json<BaseResponse<T>>, ApiError>
    where
        T: Serialize,
{
    Ok(Json(BaseResponse {
        data,
        err: Default::default(),
    }))
}

pub fn get_user(req: HttpRequest) -> String {
    req.headers().get(HeaderName::from_static("user")).unwrap().to_str().unwrap().to_string()
}