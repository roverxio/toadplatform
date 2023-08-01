use serde::Serialize;

use crate::errors::ErrorResponse;

#[derive(Serialize)]
pub struct BaseResponse<T> {
    pub data: T,
    pub err: ErrorResponse,
}
