use ethers_signers::LocalWallet;
use sqlx::{Pool, Postgres};

use crate::bundler::bundler::Bundler;
use crate::contracts::entrypoint_provider::EntryPointProvider;
use crate::contracts::simple_account_factory_provider::SimpleAccountFactoryProvider;
use crate::contracts::simple_account_provider::SimpleAccountProvider;
use crate::contracts::usdc_provider::USDCProvider;
use crate::db::dao::token_metadata_dao::TokenMetadataDao;
use crate::db::dao::transaction_dao::TransactionDao;
use crate::db::dao::wallet_dao::{User, WalletDao};
use crate::errors::ApiError;
use crate::models::transaction::transaction::Transaction;
use crate::provider::paymaster_provider::PaymasterProvider;

#[derive(Clone)]
pub struct TransferService {
    pub wallet_dao: WalletDao,
    pub transaction_dao: TransactionDao,
    pub token_metadata_dao: TokenMetadataDao,
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
    pub async fn get_status(
        db_pool: &Pool<Postgres>,
        txn_id: String,
        user: User,
    ) -> Result<Transaction, ApiError> {
        let transaction_and_exponent =
            TransactionDao::get_transaction_by_id(db_pool, txn_id, user.wallet_address).await;

        Ok(Transaction::from(transaction_and_exponent))
    }
}
