use crate::CONFIG;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs;

pub struct Utils {}

impl Utils {
    pub fn get_last_synced_time() -> i64 {
        let result = fs::read_to_string(CONFIG.get_last_sync_file_transactions());
        match result {
            Ok(time) => time.parse::<i64>().unwrap(),
            Err(_) => CONFIG.get_last_sync_time_transactions(),
        }
    }

    pub fn get_last_synced_block_number() -> i64 {
        let result = fs::read_to_string(CONFIG.get_last_sync_file_transactions());
        match result {
            Ok(number) => number.parse::<i64>().unwrap(),
            Err(_) => CONFIG.get_last_sync_block_token_transfers(),
        }
    }

    pub fn update_last_synced_time(_last_sync_time: i64) {
        // update the last sync time for the table
        unimplemented!();
    }

    pub fn update_last_synced_block(_last_synced_block: i64) {
        unimplemented!()
    }

    pub fn generate_txn_id() -> String {
        let prefix = CONFIG.get_transaction_id_prefix();
        let id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        format!("{}_{}", prefix, id).to_string()
    }
}
