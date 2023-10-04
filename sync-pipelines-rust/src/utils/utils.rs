use crate::CONFIG;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub struct Utils {}

impl Utils {
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
