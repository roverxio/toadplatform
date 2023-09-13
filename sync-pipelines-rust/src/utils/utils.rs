use crate::CONFIG;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fs;
use std::fs::File;
use std::io::Write;

pub struct Utils {}

impl Utils {
    pub fn get_last_synced_block_transactions() -> i64 {
        let result = fs::read_to_string(CONFIG.get_last_sync_file_transactions());
        match result {
            Ok(time) => time.parse::<i64>().unwrap(),
            Err(_) => CONFIG.get_last_sync_block_transactions(),
        }
    }

    pub fn get_last_synced_block_token_transfers() -> i64 {
        let result = fs::read_to_string(CONFIG.get_last_sync_file_token_transfers());
        match result {
            Ok(number) => number.parse::<i64>().unwrap(),
            Err(_) => CONFIG.get_last_sync_block_token_transfers(),
        }
    }

    pub fn update_last_synced_block_transactions(last_sync_time: i64) {
        let mut file = File::create(CONFIG.get_last_sync_file_transactions())
            .expect("Unable to create last sync file token_transfers");
        file.write(last_sync_time.to_string().as_bytes())
            .expect("Unable to write to last sync file token_transfers");
    }

    pub fn update_last_synced_block_token_transfers(last_sync_block: i64) {
        let mut file = File::create(CONFIG.get_last_sync_file_token_transfers())
            .expect("Unable to create last sync file token_transfers");
        file.write(last_sync_block.to_string().as_bytes())
            .expect("Unable to write to last sync file token_transfers");
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
