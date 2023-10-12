use actix_web::rt::spawn;
use bigdecimal::{BigDecimal, ToPrimitive};
use ethers::abi::{encode, Tokenizable};
use ethers::types::{Address, Bytes, U256};
use ethers_signers::Signer;
use sqlx::{Pool, Postgres};
use std::str::FromStr;

use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::contracts::usdc_provider::USDCProvider;
use crate::contracts::verifying_paymaster_provider::VerifyingPaymasterProvider;
use crate::db::dao::{
    TokenMetadataDao, TransactionDao, TransactionMetadata, User, UserOperationDao, UserTransaction,
};
use crate::errors::{ProviderError, TransactionError, TransferError};
use crate::models::contract_interaction::UserOperation;
use crate::models::transaction::Transaction;
use crate::models::transfer::{
    Status, TransactionResponse, TransferInitResponse, TransferResponse,
};
use crate::models::Currency;
use crate::models::TransactionType;
use crate::provider::bundler::{estimate_gas, submit_transaction};
use crate::provider::helpers::{generate_txn_id, get_explorer_url};
use crate::provider::listeners::user_op_event_listener;
use crate::provider::Web3Client;
use crate::CONFIG;

#[derive(Clone)]
pub struct TransferService;

impl TransferService {
    pub async fn init(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        to: String,
        value: String,
        currency: String,
        user: User,
    ) -> Result<TransferInitResponse, TransferError> {
        if user.wallet_address.is_empty() {
            return Err(TransferError::NotFound);
        }
        let user_txn =
            Self::get_user_transaction(&to, &value, &currency, user.wallet_address.clone());
        let mut user_op0 = UserOperation::new();
        user_op0.call_data(Self::get_call_data(pool, provider, to, value, currency).await?);
        if !user.deployed {
            user_op0.init_code(
                SimpleAccountFactoryProvider::get_factory_address(provider),
                SimpleAccountFactoryProvider::create_account(
                    provider,
                    user.owner_address.parse().unwrap(),
                    U256::from(user.salt.to_u64().unwrap()),
                )?,
            );
        }

        let wallet_address: Address = user.wallet_address.parse().unwrap();
        let valid_until: u64 = 3735928559;
        let valid_after: u64 = 4660;
        let data = encode(&vec![valid_until.into_token(), valid_after.into_token()]);
        user_op0
            .paymaster_and_data(
                data.clone(),
                CONFIG.get_chain().verifying_paymaster_address.clone(),
                None,
            )
            .nonce(
                EntryPointProvider::get_nonce(provider, wallet_address)
                    .await?
                    .low_u64(),
            )
            .sender(wallet_address.clone());

        user_op0.signature(Bytes::from(
            Web3Client::get_verifying_paymaster_wallet()
                .sign_typed_data(&user_op0)
                .await
                .map_err(|err| TransferError::Provider(err.to_string()))?
                .to_vec(),
        ));

        let singed_hash =
            Self::get_signed_hash(provider, user_op0.clone(), valid_until, valid_after).await?;
        user_op0.paymaster_and_data(
            data.clone(),
            CONFIG.get_chain().verifying_paymaster_address,
            Some(singed_hash),
        );

        let (gas_price, priority_fee) = provider.estimate_eip1559_fees().await?;
        user_op0
            .max_priority_fee_per_gas(priority_fee)
            .max_fee_per_gas(gas_price);
        let estimated_gas = estimate_gas(user_op0.clone()).await?;

        user_op0
            .call_gas_limit(estimated_gas.call_gas_limit)
            .verification_gas_limit(estimated_gas.verification_gas_limit)
            .pre_verification_gas(estimated_gas.pre_verification_gas);
        println!("user_op0: {:?}", user_op0);

        let singed_hash =
            Self::get_signed_hash(provider, user_op0.clone(), valid_until, valid_after).await?;
        user_op0.paymaster_and_data(
            data,
            CONFIG.get_chain().verifying_paymaster_address,
            Some(singed_hash),
        );

        let user_op_hash = user_op0.hash(
            CONFIG.get_chain().entrypoint_address,
            CONFIG.get_chain().chain_id,
        );
        TransactionDao::create_user_transaction(pool, user_txn.clone()).await?;
        UserOperationDao::create_user_operation(
            pool,
            user_txn.transaction_id.clone(),
            user_op0.clone(),
            Status::INITIATED.to_string(),
        )
        .await?;

        Ok(TransferInitResponse {
            msg_hash: user_op_hash,
            status: user_txn.status,
            transaction_id: user_txn.transaction_id,
        })
    }

    pub async fn execute(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        transaction_id: String,
        signature: Bytes,
        user: User,
    ) -> Result<TransferResponse, TransferError> {
        if user.wallet_address.is_empty() {
            return Err(TransferError::NotFound);
        }
        let user_op = UserOperationDao::get_user_operation(pool, transaction_id.clone()).await?;

        if user_op.transaction_id.is_empty() || user_op.status != Status::INITIATED.to_string() {
            return Err(TransferError::TxnNotFound);
        }
        UserOperationDao::update_user_operation_status(
            pool,
            transaction_id.clone(),
            Status::PENDING.to_string(),
        )
        .await?;

        let mut user_operation = user_op.user_operation;
        user_operation.signature(signature);

        let result = submit_transaction(user_operation.clone()).await;
        if result.is_err() {
            TransactionDao::update_user_transaction(
                pool,
                transaction_id,
                None,
                Status::FAILED.to_string(),
            )
            .await?;
            return Err(TransferError::Provider(
                "Failed to submit transaction".to_string(),
            ));
        }

        TransactionDao::update_user_transaction(
            pool,
            transaction_id.clone(),
            None,
            Status::SUBMITTED.to_string(),
        )
        .await?;
        spawn(user_op_event_listener(
            pool.clone(),
            provider.clone(),
            user_operation.hash(
                CONFIG.get_chain().entrypoint_address,
                CONFIG.get_chain().chain_id,
            ),
            transaction_id.clone(),
            user.deployed,
            user.external_user_id,
        ));

        Ok(TransferResponse {
            transaction: TransactionResponse {
                transaction_hash: "".to_string(),
                status: Status::PENDING.to_string(),
                explorer: get_explorer_url(""),
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
    ) -> Result<Vec<u8>, ProviderError> {
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
            Err(err) => Err(ProviderError(format!("Signing failed: {:?}", err))),
        }
    }

    async fn get_call_data(
        pool: &Pool<Postgres>,
        provider: &Web3Client,
        to: String,
        value: String,
        currency: String,
    ) -> Result<Bytes, TransferError> {
        let metadata = TokenMetadataDao::get_metadata_for_chain(
            pool,
            CONFIG.run_config.current_chain.clone(),
            Some(currency),
        )
        .await?;
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
            None => Err(TransferError::InvalidCurrency),
        }
    }
}
