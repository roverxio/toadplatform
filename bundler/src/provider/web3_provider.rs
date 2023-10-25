use ethers::abi::Abi;
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::ProviderError as EtherProviderError;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, Bytes, TransactionRequest};
use ethers::utils::format_ether;
use ethers_signers::LocalWallet;
use log::error;
use serde_json::Value;
use std::num::ParseIntError;
use std::sync::Arc;

use crate::errors::ProviderError;
use crate::PROVIDER;

#[derive(Clone)]
pub struct Web3Provider {}

#[mockall::automock]
impl Web3Provider {
    pub fn init_provider(chain_url: String) -> Provider<Http> {
        let provider = Provider::try_from(chain_url).unwrap();
        provider
    }

    pub async fn get_balance(address: Address) -> Result<String, ProviderError> {
        let result = PROVIDER.get_balance(address, None).await;
        match result {
            Ok(balance) => Ok(format_ether(balance)),
            Err(err) => Err(ProviderError(format!(
                "Get native balance failed: {:?}",
                err
            ))),
        }
    }

    pub async fn execute(
        signer: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
        to: Address,
        value: String,
        data: Bytes,
        abi: &Abi,
    ) -> Result<String, String> {
        let amount: Result<isize, ParseIntError> = value.parse();
        if amount.is_err() {
            return Err(String::from("Invalid gas value"));
        }
        let txn = TransactionRequest::new()
            .from(signer.address())
            .to(to)
            .value(amount.unwrap())
            .data(data);
        let result = signer.send_transaction(txn, None).await;
        return match result {
            Ok(transaction) => Ok(format!("{:?}", transaction.tx_hash())),
            Err(error) => match error {
                SignerMiddlewareError::SignerError(err) => {
                    error!("Signature Error: {}", err);
                    Err(String::from("Invalid signature"))
                }
                SignerMiddlewareError::MiddlewareError(middleware_error) => {
                    match middleware_error {
                        EtherProviderError::JsonRpcClientError(err) => {
                            let error = err.as_error_response();
                            match error {
                                None => Err(String::from("Json RPC error")),
                                Some(_err) => {
                                    if !_err.message.is_empty()
                                        && (_err.data.is_none()
                                            || _err.data == Some(Value::from("0x")))
                                    {
                                        error!("{}", _err.message.clone());
                                        return Err(_err.message.clone());
                                    }
                                    let error_data = _err.data.as_ref().unwrap();
                                    match error_data {
                                        Value::String(rpc_err) => {
                                            let abi_errors = abi.errors();
                                            let data_bytes =
                                                ethers::utils::hex::decode(&rpc_err[2..]).unwrap();
                                            abi_errors.for_each(|abi_error| {
                                                let _decoded_error =
                                                    abi_error.decode(&data_bytes[4..]);
                                                match _decoded_error {
                                                    Ok(_res) => {
                                                        error!(
                                                            "err_name -> {} Ok {:?}",
                                                            abi_error.name, _res
                                                        );
                                                    }
                                                    Err(_e1) => {
                                                        error!("Err: {:?}", _e1);
                                                    }
                                                }
                                            });
                                            return Err(String::from("JSON RPC error"));
                                        }
                                        _ => {
                                            error!("Non string error");
                                            Err(String::from("Non string error"))
                                        }
                                    }
                                }
                            }
                        }
                        EtherProviderError::EnsError(err) => {
                            error!("EnsError: {}", err);
                            Err(String::from("ENS name not found"))
                        }
                        EtherProviderError::EnsNotOwned(err) => {
                            error!("EnsNotOwned: {}", err);
                            Err(String::from("Invalid reverse ENS name"))
                        }
                        EtherProviderError::SerdeJson(err) => {
                            error!("SerdeJson: {}", err);
                            Err(String::from("JSON serialization error"))
                        }
                        EtherProviderError::HexError(err) => {
                            error!("HexError: {}", err);
                            Err(format!("Hex Error: {}", err))
                        }
                        EtherProviderError::HTTPError(err) => {
                            error!("HTTPError: {}", err);
                            Err(format!("HTTP Error: {}", err))
                        }
                        EtherProviderError::CustomError(err) => {
                            error!("CustomError: {}", err);
                            Err(format!("Custom Error: {}", err))
                        }
                        EtherProviderError::UnsupportedRPC => Err(String::from("Invalid RPC call")),
                        EtherProviderError::UnsupportedNodeClient => {
                            Err(String::from("Invalid Node client"))
                        }
                        EtherProviderError::SignerUnavailable => {
                            Err(String::from("Signer is not available to this provider"))
                        }
                    }
                }
                SignerMiddlewareError::NonceMissing => {
                    error!("Missing nonce");
                    Err(String::from("Missing Nonce"))
                }
                SignerMiddlewareError::GasPriceMissing => {
                    error!("Missing gas price");
                    Err(String::from("Missing gas price"))
                }
                SignerMiddlewareError::GasMissing => {
                    error!("Missing gas");
                    Err(String::from("Missing gas"))
                }
                SignerMiddlewareError::WrongSigner => {
                    error!("Wrong singer");
                    Err(String::from("Invalid signer"))
                }
                SignerMiddlewareError::DifferentChainID => {
                    error!("Invalid chain id");
                    Err(String::from("Invalid chain id"))
                }
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_provider() {
        let mock_provider = MockWeb3Provider::init_provider_context();
        mock_provider.expect().returning(|chain| {
            let provider: Provider<Http> = Provider::try_from(chain).unwrap();
            return provider;
        });

        let provider = MockWeb3Provider::init_provider("http://localhost:8545".to_string());
        assert_eq!(
            provider.url(),
            Provider::try_from("http://localhost:8545").unwrap().url()
        )
    }
}
