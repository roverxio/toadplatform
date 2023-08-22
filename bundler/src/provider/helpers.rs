use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use actix_web::http::header::HeaderName;
use actix_web::web::Json;
use actix_web::HttpRequest;
use ethers::providers::Middleware;
use ethers::types::Address;
use serde::Serialize;

use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
use crate::PROVIDER;

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
    req.headers()
        .get(HeaderName::from_static("user"))
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn get_hash(s: String) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

pub async fn contract_exists_at(address: String) -> bool {
    let formatted_address: Address = address.parse().unwrap();
    let code = PROVIDER.get_code(formatted_address, None).await.unwrap();
    !code.is_empty()
}

pub fn user_op_event_listener(_entry_point: Address, _user_op_hash: Vec<u8>) {
    // spawn a tokio::task that does the following
    // 1. open a new web socket
    // 2. subscribe for entrypoint UserOperation events with topic1 as user_op_hash
    // 3. update the user_transaction status in user_transactions table
    // 4. delete entry from user_op_hash table
    // 5. close the web socket

    // handle the errors returned by the task, if any
    unimplemented!();
}
