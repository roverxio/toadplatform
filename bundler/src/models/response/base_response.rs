use serde::Serialize;

use crate::errors::Error;

#[derive(Serialize)]
pub struct BaseResponse<T> {
    pub data: T,
    pub err: Error,
}

impl<T> BaseResponse<T> {
    pub fn init(data: T) -> BaseResponse<T>
    where
        T: Serialize,
    {
        Self {
            data,
            err: Default::default(),
        }
    }
}
