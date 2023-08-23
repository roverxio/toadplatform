use ethers::types::Address;

pub async fn user_op_event_listener(
    _entry_point: Address,
    _user_op_hash: Vec<u8>,
    _txn_id: String,
) {
    // tokio::spawn an async block that does the following
    // 1. subscribe for entrypoint UserOperation events with topic1 as user_op_hash
    //      a. wait for response till <timeout>
    //      b. in case of no response, log the timeout and return
    // 2. update the user_transaction status in user_transactions table

    // handle the errors returned by the task, if any
    unimplemented!();
}
