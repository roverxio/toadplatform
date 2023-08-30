use ethers::abi::Abi;
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::ProviderError;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, Bytes, TransactionRequest};
use ethers_signers::LocalWallet;
use log::error;
use serde_json::Value;
use std::num::ParseIntError;
use std::sync::Arc;

use crate::PROVIDER;

#[derive(Clone)]
pub struct Web3Provider {}

impl Web3Provider {
    pub fn new(chain_url: String) -> Provider<Http> {
        let provider = Provider::try_from(chain_url).unwrap();
        provider
    }

    pub async fn get_balance(address: Address) -> Result<String, String> {
        let result = PROVIDER.get_balance(address, None).await;
        if result.is_err() {
            error!("Get native balance failed: {:?}", result.err().unwrap());
            return Err(String::from("Failed to get balance"));
        }
        let wei_balance = result.unwrap().to_string();
        if wei_balance.parse::<f64>().is_err() {
            return Err(String::from("Failed to parse balance"));
        }
        Ok((wei_balance.parse::<f64>().unwrap() / 1e18).to_string())
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
                        ProviderError::JsonRpcClientError(err) => {
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
                        ProviderError::EnsError(err) => {
                            error!("EnsError: {}", err);
                            Err(String::from("ENS name not found"))
                        }
                        ProviderError::EnsNotOwned(err) => {
                            error!("EnsNotOwned: {}", err);
                            Err(String::from("Invalid reverse ENS name"))
                        }
                        ProviderError::SerdeJson(err) => {
                            error!("SerdeJson: {}", err);
                            Err(String::from("JSON serialization error"))
                        }
                        ProviderError::HexError(err) => {
                            error!("HexError: {}", err);
                            Err(format!("Hex Error: {}", err))
                        }
                        ProviderError::HTTPError(err) => {
                            error!("HTTPError: {}", err);
                            Err(format!("HTTP Error: {}", err))
                        }
                        ProviderError::CustomError(err) => {
                            error!("CustomError: {}", err);
                            Err(format!("Custom Error: {}", err))
                        }
                        ProviderError::UnsupportedRPC => Err(String::from("Invalid RPC call")),
                        ProviderError::UnsupportedNodeClient => {
                            Err(String::from("Invalid Node client"))
                        }
                        ProviderError::SignerUnavailable => {
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
