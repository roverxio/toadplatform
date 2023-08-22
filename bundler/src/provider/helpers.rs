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

pub fn generate_transaction_ids() -> String {
    unimplemented!();
}

pub fn start_user_op_event_listener(hash: String) {
    unimplemented!();
}
