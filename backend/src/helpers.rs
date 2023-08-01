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
