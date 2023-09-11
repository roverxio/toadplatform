use std::str::FromStr;

use bigdecimal::{BigDecimal, ToPrimitive};
use ethers::abi::{encode, Tokenizable};
use ethers::types::{Address, Bytes, U256};
use ethers_signers::{LocalWallet, Signer};

use crate::bundler::bundler::Bundler;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::contracts::usdc_provider::USDCProvider;
use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::db::dao::transaction_dao::{TransactionDao, TransactionMetadata, UserTransaction};
use crate::db::dao::user_operation_dao::UserOperationDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::currency::Currency;
use crate::models::transaction_type::TransactionType;
use crate::models::transfer::status::Status;
use crate::models::transfer::transaction_response::TransactionResponse;
use crate::models::transfer::transfer_init_response::TransferInitResponse;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::provider::helpers::generate_txn_id;
use crate::provider::paymaster_provider::PaymasterProvider;
use crate::provider::verifying_paymaster_helper::get_verifying_paymaster_user_operation_payload;
use crate::CONFIG;

#[derive(Clone)]
pub struct TransferServiceV2 {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub token_metadata_dao: TokenMetadataDao,
    pub user_operations_dao: UserOperationDao,
    pub usdc_provider: USDCProvider,
    pub entrypoint_provider: EntryPointProvider,
    pub simple_account_provider: SimpleAccountProvider,
    pub simple_account_factory_provider: SimpleAccountFactoryProvider,
    pub verifying_paymaster_provider: PaymasterProvider,
    pub verifying_paymaster_wallet: LocalWallet,
    pub bundler: Bundler,
}

impl TransferServiceV2 {
    pub async fn init(
        &self,
        to: String,
        value: String,
        currency: String,
        usr: &str,
    ) -> Result<TransferInitResponse, ApiError> {
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
        let user_txn =
            self.get_user_transaction(&to, &value, &currency, wallet.wallet_address.clone());
        let mut user_op0 = UserOperation::new();
        user_op0.calldata(self.get_call_data(to, value, currency).await.unwrap());
        if !wallet.deployed {
            user_op0.init_code(
                self.simple_account_factory_provider.abi.address(),
                self.simple_account_factory_provider
                    .create_account(
                        CONFIG.run_config.account_owner,
                        U256::from(wallet.salt.to_u64().unwrap()),
                    )
                    .unwrap(),
            );
        }

        let wallet_address: Address = wallet.wallet_address.parse().unwrap();
        let valid_until: u64 = 3735928559;
        let valid_after: u64 = 4660;
        let data = encode(&vec![valid_until.into_token(), valid_after.into_token()]);
        user_op0
            .paymaster_and_data(data.clone(), wallet_address.clone(), None)
            .nonce(
                self.entrypoint_provider
                    .get_nonce(wallet_address)
                    .await
                    .unwrap()
                    .low_u64(),
            )
            .sender(wallet_address.clone());

        user_op0.signature(Bytes::from(
            self.verifying_paymaster_wallet
                .sign_typed_data(&user_op0)
                .await
                .unwrap()
                .to_vec(),
        ));

        let singed_hash = self
            .get_signed_hash(user_op0.clone(), valid_until, valid_after)
            .await;
        user_op0.paymaster_and_data(
            data,
            CONFIG.get_chain().verifying_paymaster_address,
            Some(singed_hash),
        );

        let user_op_hash = user_op0.hash(
            CONFIG.get_chain().entrypoint_address,
            CONFIG.get_chain().chain_id,
        );
        self.transaction_dao
            .create_user_transaction(user_txn.clone())
            .await;
        self.user_operations_dao
            .create_user_operation(
                user_txn.transaction_id.clone(),
                user_op0.clone(),
                Status::INITIATED.to_string(),
            )
            .await;

        Ok(TransferInitResponse {
            msh_hash: Bytes::from(user_op_hash),
            status: user_txn.status,
            transaction_id: user_txn.transaction_id,
        })
    }

    pub fn execute(
        &self,
        transaction_id: String,
        _signature: Bytes,
    ) -> Result<TransferResponse, ApiError> {
        Ok(TransferResponse {
            transaction: TransactionResponse {
                transaction_hash: "0xtransaction_hash".to_string(),
                status: Status::PENDING.to_string(),
                explorer: get_explorer_url("0xtransaction_hash"),
            },
            transaction_id,
        })
    }

    fn get_transaction_metadata(&self) -> TransactionMetadata {
        let mut txn_metadata = TransactionMetadata::new();
        txn_metadata.chain(CONFIG.run_config.current_chain.clone());
        txn_metadata
    }

    fn get_user_transaction(
        &self,
        to: &String,
        value: &String,
        currency: &String,
        wallet_address: String,
    ) -> UserTransaction {
        let mut user_txn = UserTransaction::new();
        user_txn
            .user_address(wallet_address.clone())
            .transaction_id(generate_txn_id())
            .sender_address(wallet_address)
            .receiver_address(to.clone())
            .amount(BigDecimal::from_str(value).unwrap())
            .currency(currency.clone())
            .transaction_type(TransactionType::Debit.to_string())
            .status(Status::INITIATED.to_string())
            .metadata(self.get_transaction_metadata());
        user_txn
    }

    async fn get_signed_hash(
        &self,
        user_op0: UserOperation,
        valid_until: u64,
        valid_after: u64,
    ) -> Vec<u8> {
        let hash = self
            .verifying_paymaster_provider
            .get_hash(
                get_verifying_paymaster_user_operation_payload(user_op0),
                valid_until,
                valid_after,
            )
            .await
            .unwrap();
        self.verifying_paymaster_wallet
            .sign_message(hash)
            .await
            .unwrap()
            .to_vec()
    }

    async fn get_call_data(
        &self,
        to: String,
        value: String,
        currency: String,
    ) -> Result<Bytes, String> {
        match Currency::from_str(
            self.token_metadata_dao
                .get_metadata_for_chain(CONFIG.run_config.current_chain.clone(), Some(currency))
                .await[0]
                .token_type
                .clone(),
        ) {
            Some(Currency::Erc20) => Ok(self
                .simple_account_provider
                .execute(
                    CONFIG.get_chain().usdc_address,
                    0.to_string(),
                    self.usdc_provider
                        .transfer(to.parse().unwrap(), value)
                        .unwrap(),
                )
                .unwrap()),
            Some(Currency::Native) => Ok(self
                .simple_account_provider
                .execute(to.parse().unwrap(), value, Bytes::from(vec![]))
                .unwrap()),
            None => Err("Currency not found".to_string()),
        }
    }
}

pub fn get_explorer_url(txn_hash: &str) -> String {
    CONFIG.get_chain().explorer_url.clone() + &txn_hash.clone()
}
