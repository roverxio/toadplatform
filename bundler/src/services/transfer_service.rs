use actix_web::rt::spawn;
use bigdecimal::{BigDecimal, ToPrimitive};
use ethers::abi::{encode, Tokenizable};
use ethers::types::{Address, Bytes, U256};
use ethers_signers::{LocalWallet, Signer};
use sqlx::{Pool, Postgres};
use std::str::FromStr;

use crate::bundler::Bundler;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::contracts::usdc_provider::USDCProvider;
use crate::contracts::verifying_paymaster_provider::VerifyingPaymasterProvider;
use crate::db::dao::{
    TokenMetadataDao, TransactionDao, TransactionMetadata, User, UserOperationDao, UserTransaction,
    WalletDao,
};
use crate::errors::errors::ApiError;
use crate::errors::TransactionError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::currency::Currency;
use crate::models::transaction::Transaction;
use crate::models::transaction_type::TransactionType;
use crate::models::transfer::status::Status;
use crate::models::transfer::transaction_response::TransactionResponse;
use crate::models::transfer::transfer_init_response::TransferInitResponse;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::provider::helpers::{generate_txn_id, get_explorer_url};
use crate::provider::listeners::user_op_event_listener;
use crate::provider::paymaster_provider::PaymasterProvider;
use crate::CONFIG;

#[derive(Clone)]
pub struct TransferService {
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
    pub scw_owner_wallet: LocalWallet,
    pub bundler: Bundler,
}

impl TransferService {
    pub async fn init(
        &self,
        to: String,
        value: String,
        currency: String,
        user: User,
    ) -> Result<TransferInitResponse, ApiError> {
        if user.wallet_address.is_empty() {
            return Err(ApiError::NotFound("Wallet not found".to_string()));
        }
        let user_txn =
            self.get_user_transaction(&to, &value, &currency, user.wallet_address.clone());
        let mut user_op0 = UserOperation::new();
        user_op0.calldata(self.get_call_data(to, value, currency).await.unwrap());
        if !user.deployed {
            user_op0.init_code(
                self.simple_account_factory_provider.abi.address(),
                self.simple_account_factory_provider
                    .create_account(
                        user.owner_address.parse().unwrap(),
                        U256::from(user.salt.to_u64().unwrap()),
                    )
                    .unwrap(),
            );
        }

        let wallet_address: Address = user.wallet_address.parse().unwrap();
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
        TransactionDao::create_user_transaction(&self.transaction_dao.pool, user_txn.clone())
            .await
            .map_err(|err| ApiError::InternalServer(err))?;
        UserOperationDao::create_user_operation(
            &self.user_operations_dao.pool,
            user_txn.transaction_id.clone(),
            user_op0.clone(),
            Status::INITIATED.to_string(),
        )
        .await
        .map_err(|err| ApiError::InternalServer(err))?;

        Ok(TransferInitResponse {
            msg_hash: user_op_hash,
            status: user_txn.status,
            transaction_id: user_txn.transaction_id,
        })
    }

    pub async fn execute(
        &self,
        transaction_id: String,
        signature: Bytes,
        user: User,
    ) -> Result<TransferResponse, ApiError> {
        if user.wallet_address.is_empty() {
            return Err(ApiError::NotFound("Wallet not found".to_string()));
        }
        let user_op = self
            .user_operations_dao
            .get_user_operation(transaction_id.clone())
            .await;

        if user_op.transaction_id == "".to_string()
            || user_op.status != Status::INITIATED.to_string()
        {
            return Err(ApiError::NotFound("Transaction not found".to_string()));
        }
        if !self
            .user_operations_dao
            .update_user_operation_status(transaction_id.clone(), Status::PENDING.to_string())
            .await
        {
            return Err(ApiError::BadRequest(
                "Failed to update user operation status".to_string(),
            ));
        }

        let mut user_operation = user_op.user_operation;
        user_operation.signature(signature);

        let result = self
            .bundler
            .submit(user_operation.clone(), CONFIG.run_config.account_owner)
            .await;
        if result.is_err() {
            self.transaction_dao
                .update_user_transaction(transaction_id, None, Status::FAILED.to_string())
                .await;
            return Err(ApiError::BadRequest(result.err().unwrap()));
        }
        let txn_hash = result.unwrap();
        self.transaction_dao
            .update_user_transaction(transaction_id.clone(), None, Status::PENDING.to_string())
            .await;
        if !user.deployed {
            self.wallet_dao
                .update_wallet_deployed(user.external_user_id)
                .await;
        }

        spawn(user_op_event_listener(
            self.transaction_dao.clone(),
            self.entrypoint_provider.clone(),
            user_operation.hash(
                CONFIG.get_chain().entrypoint_address,
                CONFIG.get_chain().chain_id,
            ),
            transaction_id.clone(),
        ));
        self.user_operations_dao
            .update_user_operation_status(transaction_id.clone(), Status::SUCCESS.to_string())
            .await;

        Ok(TransferResponse {
            transaction: TransactionResponse {
                transaction_hash: txn_hash.clone(),
                status: Status::PENDING.to_string(),
                explorer: get_explorer_url(&txn_hash),
            },
            transaction_id,
        })
    }

    pub async fn get_status(
        pool: &Pool<Postgres>,
        txn_id: String,
        user: User,
    ) -> Result<Transaction, TransactionError> {
        let transaction =
            TransactionDao::get_transaction_by_id(pool, txn_id, user.wallet_address).await?;

        Ok(Transaction::from(transaction))
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
                VerifyingPaymasterProvider::get_verifying_paymaster_user_operation_payload(
                    user_op0,
                ),
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
