use std::str::FromStr;
use std::sync::Arc;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::types::{Address, Bytes, TransactionRequest, U256};
use ethers_signers::{LocalWallet, Signer};

use crate::CONFIG;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::provider::entrypoint_helper::{EntryPoint, get_entry_point_user_operation_payload};
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
    pub signing_client: SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
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

        println!("wallet_address: {:?}", wallet);
        let wallet_address: Address = wallet.wallet_address.parse().unwrap();
        println!("entry point address {}", self.entrypoint_provider.address());
        let nonce = self.entrypoint_provider.get_nonce(wallet_address, U256::zero()).await.unwrap();

        let user_operation = UserOperation {
            sender: wallet.wallet_address.parse().unwrap(),
            nonce,
            init_code,
            calldata,
            call_gas_limit: CONFIG.default_gas.call_gas_limit,
            verification_gas_limit: CONFIG.default_gas.verification_gas_limit,
            pre_verification_gas: CONFIG.default_gas.pre_verification_gas,
            max_fee_per_gas: CONFIG.default_gas.max_fee_per_gas,
            max_priority_fee_per_gas: CONFIG.default_gas.max_priority_fee_per_gas,
            paymaster_and_data: Bytes::from(CONFIG.chains[&CONFIG.current_chain].verifying_paymaster_address.as_bytes().to_vec()),
            signature: Default::default(),
        };
        println!("UserOperation: {:?}", user_operation);
        // sign user_operation using ecdsa
        let signed_user_operation = UserOperation {
            signature: Bytes::from(self.verifying_paymaster_signer.sign_typed_data(&user_operation).await.unwrap().to_vec()),
            ..user_operation
        };

        println!("SignedUserOperation: {:?}", signed_user_operation);

        let hash = self.verifying_paymaster_provider.get_hash(get_verifying_paymaster_user_operation_payload(signed_user_operation.clone()), u64::from_str("3900490984").unwrap(), u64::from_str("31536000").unwrap()).await.unwrap();

        // replace paymaster_and_data with hash
        let user_op = UserOperation {
            paymaster_and_data: Bytes::from(hash),
            ..signed_user_operation
        };
        let signed_user_op = UserOperation {
            signature: Bytes::from(self.verifying_paymaster_signer.sign_typed_data(&user_op).await.unwrap().to_vec()),
            ..user_op
        };
        let x = self.entrypoint_provider.handle_ops(vec![get_entry_point_user_operation_payload(signed_user_op)], CONFIG.account_owner).calldata().unwrap();
        let tx = TransactionRequest::new().from(CONFIG.account_owner).to(CONFIG.chains[&CONFIG.current_chain].entrypoint_address).value(0).data(x.clone());
        let response = self.signing_client.send_transaction(tx, None).await.unwrap();
        println!("response: {:?}", response);
        Ok(TransactionResponse {
            transaction_hash: "hash".to_string(),
            status: "success".to_string(),
            explorer: "no".to_string(),
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
