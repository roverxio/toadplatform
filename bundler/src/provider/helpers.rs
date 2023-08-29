use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use actix_web::http::header::HeaderName;
use actix_web::web::Json;
use actix_web::HttpRequest;
use ethers::providers::Middleware;
use ethers::types::Address;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Serialize;

use crate::db::dao::transaction_dao::UserTransactionWithExponent;
use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
use crate::models::transaction::transaction::{Amount, Metadata, Transaction, UserInfo};
use crate::{CONFIG, PROVIDER};

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

pub fn generate_txn_id() -> String {
    let prefix = &CONFIG.run_config.transaction_id_prefix;
    let id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    format!("{}_{}", prefix, id).to_string()
}

pub fn get_explorer_url(txn_hash: &str) -> String {
    CONFIG.get_chain().explorer_url.clone() + &txn_hash.clone()
}

pub fn get_transaction(transaction_and_exponent: UserTransactionWithExponent) -> Transaction {
    let transaction = transaction_and_exponent.user_transaction;
    Transaction {
        transaction_id: transaction.transaction_id,
        amount: Amount {
            currency: transaction.currency,
            value: transaction.amount,
            exponent: transaction_and_exponent.exponent,
        },
        metadata: Metadata {
            chain: transaction.metadata.chain,
            gas: Amount::default(),
            transaction_hash: transaction.metadata.transaction_hash.clone(),
            timestamp: transaction.updated_at,
            explorer_url: get_explorer_url(&transaction.metadata.transaction_hash),
            status: transaction.status,
        },
        from: UserInfo {
            address: transaction.from_address,
            name: transaction.metadata.from_name,
        },
        id: transaction.id,
        to: UserInfo {
            address: transaction.to_address,
            name: transaction.metadata.to_name,
        },
        transaction_type: transaction.transaction_type,
    }
}
