use crate::utils::table::Table;
use crate::CONFIG;
use chrono::{DateTime, Utc};

pub struct Utils {}

const DATETIME_PATTERN: &str = "%Y-%m-%d %H:%M:%S %z";

impl Utils {
    pub fn get_last_synced_time(table: Table) -> DateTime<Utc> {
        match table {
            Table::TokenTransfers => DateTime::parse_from_str(
                CONFIG.get_last_sync_time_token_transfers(),
                DATETIME_PATTERN,
            )
            .unwrap()
            .with_timezone(&Utc),
            Table::Transactions => {
                DateTime::parse_from_str(CONFIG.get_last_sync_time_transactions(), DATETIME_PATTERN)
                    .unwrap()
                    .with_timezone(&Utc)
            }
        }
    }

    pub fn update_last_synced_time(_table: String, _last_sync_time: DateTime<Utc>) {
        // update the last sync time for the table
        unimplemented!();
    }

    pub fn generate_txn_id() -> String {
        // generate and return random id
        unimplemented!();
    }
}
