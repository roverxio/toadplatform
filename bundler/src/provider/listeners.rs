use ethers::abi::RawLog;
use ethers::providers::Middleware;
use ethers::types::{Filter, H256};
use sqlx::{Pool, Postgres};

use crate::db::dao::TransactionDao;
use crate::models::transfer::Status::{FAILED, SUCCESS};
use crate::provider::Web3Client;
use crate::{CONFIG, PROVIDER};

pub async fn user_op_event_listener(
    pool: Pool<Postgres>,
    client: Web3Client,
    user_op_hash: [u8; 32],
    txn_id: String,
) -> Result<(), String> {
    let provider = client.get_entrypoint_provider();
    let event = provider
        .abi()
        .event("UserOperationEvent")
        .map_err(|_| String::from("Failed to get event"))?;

    let filter = Filter::new()
        .address(CONFIG.get_chain().entrypoint_address)
        .topic0(event.signature())
        .topic1(H256::from(user_op_hash));

    let log_data = loop {
        let logs = PROVIDER.get_logs(&filter).await.unwrap();
        if logs.len() > 0 {
            break logs[0].clone();
        }
    };

    let txn_hash = format!("{:?}", log_data.transaction_hash.unwrap());

    let log = event
        .parse_log(RawLog {
            topics: log_data.topics,
            data: log_data.data.to_vec(),
        })
        .unwrap();

    let success_param = log
        .params
        .into_iter()
        .find(|param| param.name == "success")
        .unwrap();
    let success = success_param.value.into_bool().unwrap();

    let status = if success { SUCCESS } else { FAILED };

    TransactionDao::update_user_transaction(&pool, txn_id, Some(txn_hash), status.to_string())
        .await
        .map_err(|_| String::from("Listener: Failed to update database"))
}
