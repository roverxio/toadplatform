use crate::utils::table::Table;
use std::fs;
use std::fs::File;
use std::io::Write;

pub struct LastSync {}

impl LastSync {
    pub fn get_last_synced_block(table: Table) -> i64 {
        let result = fs::read_to_string(table.get_path());
        match result {
            Ok(block) => block.parse::<i64>().unwrap(),
            Err(_) => table.get_block_number(),
        }
    }

    pub fn update_last_synced_block(table: Table, last_sync_block: i64) -> Result<(), String> {
        let mut file = File::create(table.get_path())
            .map_err(|_| format!("Unable to create last sync file {}", table.to_string()))?;
        file.write(last_sync_block.to_string().as_bytes())
            .map_err(|_| format!("Unable to write to last sync file {}", table.to_string()))?;
        Ok(())
    }
}
