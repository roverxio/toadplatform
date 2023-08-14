use serde::Serialize;

use crate::errors::Error;

#[derive(Serialize)]
pub struct BaseResponse<T> {
    pub data: T,
    pub err: Error,
}
