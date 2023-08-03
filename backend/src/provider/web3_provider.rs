use ethers::{
    providers::{Http, Provider},
};

pub struct Web3Provider {}

impl Web3Provider {
    pub fn new(chain_url: String) -> Provider<Http> {
        let provider = Provider::try_from(chain_url).unwrap();
        provider
    }
}
