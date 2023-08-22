use std::str::FromStr;

use crate::bundler::bundler::Bundler;
use ethers::abi::{encode, Token, Tokenizable};
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use ethers_signers::{LocalWallet, Signer};
use log::info;

use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactory;
use crate::contracts::simple_account_provider::SimpleAccount;
use crate::contracts::usdc_provider::ERC20;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::transfer::transaction_response::TransactionResponse;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::provider::verifying_paymaster_helper::{
    get_verifying_paymaster_user_operation_payload, VerifyingPaymaster,
};
use crate::CONFIG;

#[derive(Clone)]
pub struct TransferService {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub usdc_provider: ERC20<Provider<Http>>,
    pub entrypoint_provider: EntryPointProvider,
    pub simple_account_provider: SimpleAccount<Provider<Http>>,
    pub simple_account_factory_provider: SimpleAccountFactory<Provider<Http>>,
    pub verifying_paymaster_provider: VerifyingPaymaster<Provider<Http>>,
    pub verifying_paymaster_signer: LocalWallet,
    pub wallet_singer: LocalWallet,
    pub bundler: Bundler,
}

impl TransferService {
    pub async fn transfer_funds(
        &self,
        request: TransferRequest,
        usr: &str,
    ) -> Result<TransferResponse, ApiError> {
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
        if request.metadata.currency.to_lowercase() == "native" {
            calldata = self.transfer_native(request.receiver, request.value);
        } else if request.metadata.currency.to_lowercase() == "usdc" {
            calldata =
                self.transfer_usdc(self.get_transfer_payload(request.receiver, request.value));
        } else {
            return Err(ApiError::NotFound("Currency not found".to_string()));
        }
        if !wallet.deployed {
            let create_account_payload = self
                .simple_account_factory_provider
                .create_account(
                    CONFIG.run_config.account_owner,
                    U256::from_dec_str(&wallet.salt).unwrap(),
                )
                .calldata()
                .unwrap();
            init_code = Bytes::from(
                [
                    CONFIG.get_chain().simple_account_factory_address.as_bytes(),
                    create_account_payload.as_ref(),
                ]
                .concat(),
            );
        }

        let wallet_address: Address = wallet.wallet_address.parse().unwrap();
        let nonce = self
            .entrypoint_provider
            .get_nonce(wallet_address)
            .await
            .unwrap();

        let valid_until = u64::from_str("3735928559").unwrap();
        let valid_after = u64::from_str("4660").unwrap();
        let params: Vec<Token> = vec![valid_until.into_token(), valid_after.into_token()];
        let data = encode(&params);
        let paymaster_and_data = [
            CONFIG.get_chain().verifying_paymaster_address.as_bytes(),
            data.as_ref(),
            &vec![0u8; 65],
        ]
        .concat();

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
            signature: Bytes::from(
                self.verifying_paymaster_signer
                    .sign_typed_data(&user_op0)
                    .await
                    .unwrap()
                    .to_vec(),
            ),
            ..user_op0
        };

        let hash = self
            .verifying_paymaster_provider
            .get_hash(
                get_verifying_paymaster_user_operation_payload(usr_op1.clone()),
                valid_until,
                valid_after,
            )
            .await
            .unwrap();
        let singed_hash = self
            .verifying_paymaster_signer
            .sign_message(&hash)
            .await
            .unwrap()
            .to_vec();
        let paymaster_and_data_with_sign = [
            CONFIG.get_chain().verifying_paymaster_address.as_bytes(),
            data.as_ref(),
            &singed_hash,
        ]
        .concat();

        // replace paymaster_and_data with hash
        let user_op2 = UserOperation {
            paymaster_and_data: Bytes::from(paymaster_and_data_with_sign),
            ..usr_op1
        };

        let signature = Bytes::from(
            self.wallet_singer
                .sign_message(user_op2.hash(
                    CONFIG.get_chain().entrypoint_address,
                    CONFIG.get_chain().chain_id,
                ))
                .await
                .unwrap()
                .to_vec(),
        );

        let user_op3 = UserOperation {
            signature,
            ..user_op2
        };

        let result = self
            .bundler
            .submit(user_op3, CONFIG.run_config.account_owner)
            .await;
        if result.is_err() {
            return Err(ApiError::BadRequest(result.err().unwrap()));
        }

        let txn_hash = result.unwrap();
        info!("Transaction sent successfully. Hash: {:?}", txn_hash);
        self.transaction_dao
            .create_transaction(txn_hash.clone(), wallet.wallet_address.clone())
            .await;
        if !wallet.deployed {
            self.wallet_dao
                .update_wallet_deployed(usr.to_string())
                .await;
        }

        Ok(TransferResponse {
            transaction: TransactionResponse {
                transaction_hash: txn_hash.clone(),
                status: "pending".to_string(),
                explorer: CONFIG.get_chain().explorer_url.clone() + &txn_hash.clone(),
            },
        })
    }

    fn get_transfer_payload(&self, receiver: String, amount: String) -> Bytes {
        let target: Address = receiver.parse().unwrap();
        let value: f64 = amount.parse().unwrap();
        let usdc_amount = value * 1e6;
        self.usdc_provider
            .transfer(target, U256::from(usdc_amount as u64))
            .calldata()
            .unwrap()
    }

    fn transfer_usdc(&self, transfer_payload: Bytes) -> Bytes {
        self.simple_account_provider
            .execute(
                CONFIG.get_chain().usdc_address,
                U256::zero(),
                transfer_payload,
            )
            .calldata()
            .unwrap()
    }

    fn transfer_native(&self, receiver: String, amount: String) -> Bytes {
        let value: f64 = amount.parse().unwrap();
        let wei = value * 1e18;
        self.simple_account_provider
            .execute(
                receiver.parse().unwrap(),
                U256::from(wei as u64),
                Bytes::from(vec![]),
            )
            .calldata()
            .unwrap()
    }
}
