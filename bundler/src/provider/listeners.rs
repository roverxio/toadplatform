use crate::db::dao::transaction_dao::TransactionDao;
use crate::PROVIDER;
use ethers::abi::decode;
use ethers::abi::ParamType::{Bool, Uint};
use ethers::providers::Middleware;
use ethers::types::{Address, Filter, H256};

const USER_OPERATION_EVENT: &str =
    "0x49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f";

pub async fn user_op_event_listener(
    transaction_dao: TransactionDao,
    entry_point: Address,
    user_op_hash: [u8; 32],
    txn_id: String,
) {
    let filter = Filter::new()
        .address(entry_point)
        .topic0(H256::from(USER_OPERATION_EVENT.parse::<H256>().unwrap()))
        .topic1(H256::from(user_op_hash));

    let logs = loop {
        let logs = PROVIDER.get_logs(&filter).await.unwrap();
        if logs.iter().len() > 0 {
            break logs;
        }
    };

    let txn_hash = logs[0].transaction_hash.unwrap();
    let data = decode(&[Uint(256), Bool, Uint(256), Uint(256)], &*logs[0].data)
        .unwrap()
        .get(1)
        .unwrap()
        .clone()
        .into_bool()
        .unwrap();
    let status = if data {
        "success".to_string()
    } else {
        "failed".to_string()
    };

    transaction_dao
        .update_user_transactions(txn_id, txn_hash, status)
        .await;
}
