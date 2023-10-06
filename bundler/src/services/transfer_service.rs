use actix_web::rt::spawn;
use bigdecimal::{BigDecimal, ToPrimitive};
use ethers::abi::{encode, Tokenizable};
use ethers::types::{Address, Bytes, U256};
use ethers_signers::Signer;
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
use crate::provider::Web3Client;
use crate::CONFIG;

#[derive(Clone)]
pub struct TransferService {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub user_operations_dao: UserOperationDao,
    pub entrypoint_provider: EntryPointProvider,
    pub bundler: Bundler,
}

impl TransferService {
    pub async fn init(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        to: String,
        value: String,
        currency: String,
        user: User,
    ) -> Result<TransferInitResponse, ApiError> {
        if user.wallet_address.is_empty() {
            return Err(ApiError::NotFound("Wallet not found".to_string()));
        }
        let user_txn =
            Self::get_user_transaction(&to, &value, &currency, user.wallet_address.clone());
        let mut user_op0 = UserOperation::new();
        user_op0.calldata(
            Self::get_call_data(pool, provider, to, value, currency)
                .await
                .unwrap(),
        );
        if !user.deployed {
            user_op0.init_code(
                SimpleAccountFactoryProvider::get_factory_address(provider),
                SimpleAccountFactoryProvider::create_account(
                    provider,
                    user.owner_address.parse().unwrap(),
                    U256::from(user.salt.to_u64().unwrap()),
                )
                .map_err(|err| ApiError::InternalServer(err))?,
            );
        }

        let wallet_address: Address = user.wallet_address.parse().unwrap();
        let valid_until: u64 = 3735928559;
        let valid_after: u64 = 4660;
        let data = encode(&vec![valid_until.into_token(), valid_after.into_token()]);
        user_op0
            .paymaster_and_data(data.clone(), wallet_address.clone(), None)
            .nonce(
                EntryPointProvider::get_nonce(provider, wallet_address)
                    .await
                    .unwrap()
                    .low_u64(),
            )
            .sender(wallet_address.clone());

        user_op0.signature(Bytes::from(
            Web3Client::get_verifying_paymaster_wallet()
                .sign_typed_data(&user_op0)
                .await
                .unwrap()
                .to_vec(),
        ));

        let singed_hash =
            Self::get_signed_hash(provider, user_op0.clone(), valid_until, valid_after)
                .await
                .map_err(|err| ApiError::InternalServer(err))?;
        user_op0.paymaster_and_data(
            data,
            CONFIG.get_chain().verifying_paymaster_address,
            Some(singed_hash),
        );

        let user_op_hash = user_op0.hash(
            CONFIG.get_chain().entrypoint_address,
            CONFIG.get_chain().chain_id,
        );
        TransactionDao::create_user_transaction(pool, user_txn.clone())
            .await
            .map_err(|err| ApiError::InternalServer(err))?;
        UserOperationDao::create_user_operation(
            pool,
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

    fn get_transaction_metadata() -> TransactionMetadata {
        let mut txn_metadata = TransactionMetadata::new();
        txn_metadata.chain(CONFIG.run_config.current_chain.clone());
        txn_metadata
    }

    fn get_user_transaction(
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
            .metadata(Self::get_transaction_metadata());
        user_txn
    }

    async fn get_signed_hash(
        provider: &Web3Client,
        user_op0: UserOperation,
        valid_until: u64,
        valid_after: u64,
    ) -> Result<Vec<u8>, String> {
        let hash = VerifyingPaymasterProvider::get_hash(
            provider,
            VerifyingPaymasterProvider::get_verifying_paymaster_user_operation_payload(user_op0),
            valid_until,
            valid_after,
        )
        .await?;
        let result = Web3Client::get_verifying_paymaster_wallet()
            .sign_message(hash)
            .await;
        match result {
            Ok(signature) => Ok(signature.to_vec()),
            Err(err) => Err(format!("Signing failed: {:?}", err)),
        }
    }

    async fn get_call_data(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        to: String,
        value: String,
        currency: String,
    ) -> Result<Bytes, String> {
        let metadata = TokenMetadataDao::get_metadata_for_chain(
            pool,
            CONFIG.run_config.current_chain.clone(),
            Some(currency),
        )
        .await
        .map_err(|_| String::from("Failed to get metadata"))?;
        match Currency::from_str(metadata[0].token_type.clone()) {
            Some(Currency::Erc20) => Ok(SimpleAccountProvider::execute(
                provider,
                CONFIG.get_chain().usdc_address,
                0.to_string(),
                USDCProvider::transfer(provider, to.parse().unwrap(), value)?,
            )?),
            Some(Currency::Native) => Ok(SimpleAccountProvider::execute(
                provider,
                to.parse().unwrap(),
                value,
                Bytes::from(vec![]),
            )?),
            None => Err(String::from("Currency not found")),
        }
    }
}
