use ethers::abi::RawLog;
use ethers::providers::Middleware;
use ethers::types::{Filter, H256};

use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::user_operation_dao::UserOperationDao;
use crate::db::dao::wallet_dao::WalletDao;
use crate::models::transfer::status::Status::{FAILED, SUCCESS};
use crate::{CONFIG, PROVIDER};

pub async fn user_op_event_listener(
    transaction_dao: TransactionDao,
    wallet_dao: WalletDao,
    user_operations_dao: UserOperationDao,
    entrypoint_provider: EntryPointProvider,
    user_op_hash: [u8; 32],
    txn_id: String,
    wallet_deployed: bool,
    external_user_id: String,
) {
    let event = entrypoint_provider
        .abi()
        .event("UserOperationEvent")
        .unwrap();

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

    transaction_dao
        .update_user_transaction(txn_id.clone(), Some(txn_hash), status.to_string())
        .await;

    user_operations_dao
        .update_user_operation_status(txn_id, status.to_string())
        .await;

    if success && !wallet_deployed {
        wallet_dao.update_wallet_deployed(external_user_id).await;
    }
}
