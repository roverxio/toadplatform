use ethers::abi::{encode, Tokenizable};
use ethers::providers::{Http, Provider};
use ethers::types::{Address, Bytes, U256};
use ethers_signers::{LocalWallet, Signer};
use log::info;

use crate::bundler::bundler::Bundler;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::contracts::usdc_provider::USDCProvider;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::transfer::transaction_response::TransactionResponse;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::provider::verifying_paymaster_helper::{
    get_verifying_paymaster_user_operation_payload, VerifyingPaymaster,
};
use crate::CONFIG;

#[derive(Clone)]
pub struct TransferService {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub usdc_provider: USDCProvider,
    pub entrypoint_provider: EntryPointProvider,
    pub simple_account_provider: SimpleAccountProvider,
    pub simple_account_factory_provider: SimpleAccountFactoryProvider,
    pub verifying_paymaster_provider: VerifyingPaymaster<Provider<Http>>,
    pub verifying_paymaster_signer: LocalWallet,
    pub wallet_singer: LocalWallet,
    pub bundler: Bundler,
}

impl TransferService {
    pub async fn transfer_funds(
        &self,
        to: String,
        value: String,
        currency: String,
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
        let mut user_op0 = UserOperation::new();
        user_op0.calldata(self.get_call_data(to, value, currency).unwrap());
        if !wallet.deployed {
            user_op0.init_code(
                self.simple_account_factory_provider.abi.address(),
                self.simple_account_factory_provider
                    .create_account(
                        CONFIG.run_config.account_owner,
                        U256::from_dec_str(&wallet.salt).unwrap(),
                    )
                    .unwrap(),
            );
        }

        let wallet_address: Address = wallet.wallet_address.parse().unwrap();
        let valid_until: u64 = 3735928559;
        let valid_after: u64 = 4660;
        let data = encode(&vec![valid_until.into_token(), valid_after.into_token()]);
        user_op0
            .paymaster_and_data(data.clone(), wallet_address.clone())
            .nonce(
                self.entrypoint_provider
                    .get_nonce(wallet_address)
                    .await
                    .unwrap()
                    .low_u64(),
            )
            .sender(wallet_address.clone());

        user_op0.signature(Bytes::from(
            self.verifying_paymaster_signer
                .sign_typed_data(&user_op0)
                .await
                .unwrap()
                .to_vec(),
        ));

        let hash = self
            .verifying_paymaster_provider
            .get_hash(
                get_verifying_paymaster_user_operation_payload(user_op0.clone()),
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
        user_op0.signed_paymaster_and_data(
            data,
            CONFIG.chains[&CONFIG.run_config.current_chain].verifying_paymaster_address,
            Bytes::from(singed_hash),
        );
        let signature = Bytes::from(
            self.wallet_singer
                .sign_message(user_op0.hash(
                    CONFIG.chains[&CONFIG.run_config.current_chain].entrypoint_address,
                    CONFIG.chains[&CONFIG.run_config.current_chain].chain_id,
                ))
                .await
                .unwrap()
                .to_vec(),
        );

        user_op0.signature(signature);
        let result = self
            .bundler
            .submit(user_op0, CONFIG.run_config.account_owner)
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
                explorer: CONFIG.chains[&CONFIG.run_config.current_chain]
                    .explorer_url
                    .clone()
                    + &txn_hash.clone(),
            },
        })
    }

    fn get_call_data(&self, to: String, value: String, currency: String) -> Result<Bytes, String> {
        if currency.to_lowercase() == "native" {
            Ok(self
                .simple_account_provider
                .execute(to.parse().unwrap(), value, Bytes::from(vec![]))
                .unwrap())
        } else if currency.to_lowercase() == "usdc" {
            Ok(self
                .simple_account_provider
                .execute(
                    to.parse().unwrap(),
                    0.to_string(),
                    self.usdc_provider
                        .transfer(to.parse().unwrap(), value)
                        .unwrap(),
                )
                .unwrap())
        } else {
            return Err("Currency not found".to_string());
        }
    }
}
