use crate::CONFIG;
use ethers::contract::abigen;
use ethers::providers::{Http, Provider};
use std::sync::Arc;

abigen!(EntryPoint, "abi/Entrypoint.json");

pub fn get_entrypoint_abi(
    current_chain: &str,
    client: Arc<Provider<Http>>,
) -> EntryPoint<Provider<Http>> {
    let contract: EntryPoint<Provider<Http>> =
        EntryPoint::new(CONFIG.chains[current_chain].entrypoint_address, client);
    contract
}
