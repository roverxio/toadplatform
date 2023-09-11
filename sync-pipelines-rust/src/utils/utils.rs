use crate::CONFIG;
use bigdecimal::num_bigint::BigInt;
use chrono::{DateTime, Utc};
use std::fs;

pub struct Utils {}

impl Utils {
    const DATETIME_PATTERN: &'static str = "%Y-%m-%d %H:%M:%S %z";

    pub fn get_last_synced_time() -> DateTime<Utc> {
        let result = fs::read_to_string(CONFIG.get_last_sync_file_transactions());
        match result {
            Ok(time) => DateTime::parse_from_str(time.as_str(), Self::DATETIME_PATTERN)
                .unwrap()
                .with_timezone(&Utc),
            Err(_) => DateTime::parse_from_str(
                CONFIG.get_last_sync_time_transactions(),
                Self::DATETIME_PATTERN,
            )
            .unwrap()
            .with_timezone(&Utc),
        }
    }

    pub fn get_last_synced_block_number() -> u64 {
        let result = fs::read_to_string(CONFIG.get_last_sync_file_transactions());
        match result {
            Ok(number) => number.parse::<u64>().unwrap(),
            Err(_) => CONFIG.get_last_sync_block_token_transfers(),
        }
    }

    pub fn update_last_synced_time(_last_sync_time: DateTime<Utc>) {
        // update the last sync time for the table
        unimplemented!();
    }

    pub fn update_last_synced_block(_last_synced_block: BigInt) {
        unimplemented!()
    }

    pub fn generate_txn_id() -> String {
        // generate and return random id
        unimplemented!();
    }
}
