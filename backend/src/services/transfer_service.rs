use std::str::FromStr;
use std::sync::Arc;
use ethers::abi::{encode, Token, Tokenizable};
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider, ProviderError};
use ethers::types::{Address, Bytes, TransactionRequest, U256};
use ethers_signers::{LocalWallet, Signer};
use serde_json::Value;

use crate::CONFIG;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::provider::entrypoint_helper::{EntryPoint, get_entry_point_user_operation_payload};
use crate::provider::http_client::HttpClient;
use crate::provider::verifying_paymaster_helper::{get_verifying_paymaster_user_operation_payload, VerifyingPaymaster};
use crate::provider::web3_provider::{ERC20, Simpleaccount, SimpleAccountFactory};

#[derive(Clone)]
pub struct TransactionService {
    pub wallet_dao: WalletDao,
    pub usdc_provider: ERC20<Provider<Http>>,
    pub entrypoint_provider: EntryPoint<Provider<Http>>,
    pub simple_account_provider: Simpleaccount<Provider<Http>>,
    pub simple_account_factory_provider: SimpleAccountFactory<Provider<Http>>,
    pub verifying_paymaster_provider: VerifyingPaymaster<Provider<Http>>,
    pub verifying_paymaster_signer: LocalWallet,
    pub wallet_singer: LocalWallet,
    pub signing_client: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    pub http_client: HttpClient,
}

impl TransactionService {
    pub async fn transfer_funds(
        &self,
        request: TransferRequest,
        usr: &str,
    ) -> Result<TransactionResponse, ApiError> {
        let user_wallet = self.wallet_dao.get_wallet(usr.to_string()).await;
        let wallet: User;
        match user_wallet {
            None => {
                return Err(ApiError::NotFound("Wallet not found".to_string()));
            }
            Some(_) => {
                wallet = user_wallet.unwrap();
            }
        }
        let calldata: Bytes;
        let mut init_code: Bytes = Bytes::from(vec![]);
        if request.metadata.currency == "native" {
            calldata = self.transfer_native(request.receiver, request.value);
        } else if request.metadata.currency == "usdc" {
            calldata = self.transfer_usdc(self.get_transfer_payload(request.receiver, request.value));
        } else {
            return Err(ApiError::NotFound("Currency not found".to_string()));
        }
        if !wallet.deployed {
            let create_account_payload = self.simple_account_factory_provider.create_account(CONFIG.account_owner, U256::from_dec_str(&wallet.salt).unwrap()).calldata().unwrap();
            init_code = Bytes::from([CONFIG.chains[&CONFIG.current_chain].simple_account_factory_address.as_bytes(), create_account_payload.as_ref()].concat());
        }

        let wallet_address: Address = wallet.wallet_address.parse().unwrap();
        let nonce = self.entrypoint_provider.get_nonce(wallet_address, U256::zero()).await.unwrap();

        let valid_until = u64::from_str("3735928559").unwrap();
        let valid_after = u64::from_str("4660").unwrap();
        let params: Vec<Token> = vec![valid_until.into_token(), valid_after.into_token()];
        let data = encode(&params);
        let paymaster_and_data = [CONFIG.chains[&CONFIG.current_chain].verifying_paymaster_address.as_bytes(), data.as_ref(), &vec![0u8; 65]].concat();

        let user_op0 = UserOperation {
            sender: wallet.wallet_address.parse().unwrap(),
            nonce: nonce.low_u64(),
            init_code,
            calldata,
            call_gas_limit: CONFIG.default_gas.call_gas_limit,
            verification_gas_limit: CONFIG.default_gas.verification_gas_limit,
            pre_verification_gas: CONFIG.default_gas.pre_verification_gas,
            max_fee_per_gas: CONFIG.default_gas.max_fee_per_gas,
            max_priority_fee_per_gas: CONFIG.default_gas.max_priority_fee_per_gas,
            paymaster_and_data: Bytes::from(paymaster_and_data),
            signature: Default::default(),
        };
        // sign user_operation using ecdsa
        let usr_op1 = UserOperation {
            signature: Bytes::from(self.verifying_paymaster_signer.sign_typed_data(&user_op0).await.unwrap().to_vec()),
            ..user_op0
        };

        let hash = self.verifying_paymaster_provider.get_hash(get_verifying_paymaster_user_operation_payload(usr_op1.clone()), valid_until, valid_after).await.unwrap();
        let singed_hash = self.verifying_paymaster_signer.sign_message(&hash).await.unwrap().to_vec();
        let paymaster_and_data_with_sign = [CONFIG.chains[&CONFIG.current_chain].verifying_paymaster_address.as_bytes(), data.as_ref(), &singed_hash].concat();

        // replace paymaster_and_data with hash
        let user_op2 = UserOperation {
            paymaster_and_data: Bytes::from(paymaster_and_data_with_sign),
            ..usr_op1
        };
        let signature = self.http_client.sign_message(user_op2.clone(), format!("{:?}", self.entrypoint_provider.address().clone()), CONFIG.chains[&CONFIG.current_chain].chain_id.clone()).await.unwrap();

        let user_op3 = UserOperation {
            signature,
            ..user_op2
        };

        let handle_ops_payload = self.entrypoint_provider.handle_ops(vec![get_entry_point_user_operation_payload(user_op3)], CONFIG.account_owner).calldata().unwrap();
        let transaction = TransactionRequest::new().from(CONFIG.account_owner).to(CONFIG.chains[&CONFIG.current_chain].entrypoint_address).value(0).data(handle_ops_payload.clone());
        let result = self.signing_client.send_transaction(transaction, None).await;

        let mut txn_hash: String = "".to_string();
        match result {
            Ok(hash) => {
                println!("Transaction sent successfully. Hash: {:?}", hash);
                txn_hash = hash.tx_hash().to_string();
                // update database to update user's "deployed" to true
                if !wallet.deployed {
                    self.wallet_dao.update_wallet_deployed(usr.to_string()).await;
                }
            }
            Err(err) => {
                match err {
                    SignerMiddlewareError::SignerError(a) => {
                        println!("Signer error: {}", a);
                    }
                    SignerMiddlewareError::MiddlewareError(b) => {
                        println!("Middleware error: {}", b);
                        match b {
                            ProviderError::JsonRpcClientError(_a) => {
                                println!("JsonRpcClientError: {}", _a);
                                let error = _a.as_error_response();
                                match error {
                                    None => {}
                                    Some(_err) => {
                                        let error_data = _err.data.as_ref().unwrap();
                                        match error_data {
                                            Value::Null => {}
                                            Value::Bool(_) => {}
                                            Value::Number(_) => {}
                                            Value::String(_str_err) => {
                                                println!("Error: {}", _str_err);
                                                let abi_errors = self.entrypoint_provider.abi().errors();
                                                let data_bytes = ethers::utils::hex::decode(&_str_err[2..]).unwrap();
                                                abi_errors.for_each(|abi_error| {
                                                    let _decoded_error = abi_error.decode(&data_bytes[4..]);
                                                    match _decoded_error {
                                                        Ok(_res) => {
                                                            println!("err_name -> {} Ok {:?}", abi_error.name, _res);
                                                        }
                                                        Err(_e1) => {
                                                            println!("Err: {:?}", _e1);
                                                        }
                                                    }
                                                });
                                                // let encoded_bytes = ethers::utils::hex::decode(&_str_err[2..]).unwrap();
                                            }
                                            Value::Array(_) => {}
                                            Value::Object(_) => {}
                                        }
                                    }
                                }
                            }
                            ProviderError::EnsError(_b) => {
                                println!("EnsError: {}", _b);
                            }
                            ProviderError::EnsNotOwned(_c) => {
                                println!("EnsNotOwned: {}", _c);
                            }
                            ProviderError::SerdeJson(_d) => {
                                println!("SerdeJson: {}", _d);
                            }
                            ProviderError::HexError(_e) => {
                                println!("HexError: {}", _e);
                            }
                            ProviderError::HTTPError(_f) => {
                                println!("HTTPError: {}", _f);
                            }
                            ProviderError::CustomError(_g) => {
                                println!("CustomError: {}", _g);
                            }
                            ProviderError::UnsupportedRPC => {}
                            ProviderError::UnsupportedNodeClient => {}
                            ProviderError::SignerUnavailable => {}
                        }
                    }
                    SignerMiddlewareError::NonceMissing => {}
                    SignerMiddlewareError::GasPriceMissing => {}
                    SignerMiddlewareError::GasMissing => {}
                    SignerMiddlewareError::WrongSigner => {}
                    SignerMiddlewareError::DifferentChainID => {}
                }
            }
        }

        Ok(TransactionResponse {
            transaction_hash: txn_hash.clone(),
            status: "pending".to_string(),
            explorer: CONFIG.chains[&CONFIG.current_chain].explorer_url.clone() + &txn_hash.clone(),
        })
    }

    fn get_transfer_payload(&self, receiver: String, amount: String) -> Bytes {
        let value = amount.parse::<U256>().unwrap();
        let target: Address = receiver.parse().unwrap();
        self.usdc_provider
            .transfer(target, value)
            .calldata()
            .unwrap()
    }

    fn transfer_usdc(&self, transfer_payload: Bytes) -> Bytes {
        self.simple_account_provider.execute(
            CONFIG.chains[&CONFIG.current_chain].usdc_address,
            U256::zero(),
            transfer_payload,
        ).calldata().unwrap()
    }

    fn transfer_native(&self, receiver: String, amount: String) -> Bytes {
        self.simple_account_provider.execute(
            receiver.parse().unwrap(),
            amount.parse::<U256>().unwrap(),
            Bytes::from(vec![]),
        ).calldata().unwrap()
    }
}
