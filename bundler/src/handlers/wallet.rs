use actix_web::web::{Data, Query, ReqData};
use actix_web::{HttpRequest, HttpResponse};
use sqlx::{Pool, Postgres};

use crate::db::dao::wallet_dao::User;
use crate::errors::{BalanceError, TransactionError, WalletError};
use crate::models::response::base_response::BaseResponse;
use crate::models::transaction::{ListTransactionsParams, PollTransactionParams};
use crate::models::wallet::BalanceRequest;
use crate::provider::helpers::get_user_wallet;
use crate::provider::Web3Client;
use crate::services::TransferService;
use crate::services::{BalanceService, WalletService};

pub async fn get_address(
    pool: Data<Pool<Postgres>>,
    provider: Data<Web3Client>,
    user: ReqData<User>,
    req: HttpRequest,
) -> Result<HttpResponse, WalletError> {
    let wallet_address = WalletService::get_wallet_address(
        pool.get_ref(),
        provider.get_ref(),
        user.into_inner(),
        get_user_wallet(req),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(wallet_address)))
}

pub async fn get_balance(
    pool: Data<Pool<Postgres>>,
    provider: Data<Web3Client>,
    body: Query<BalanceRequest>,
    user: ReqData<User>,
) -> Result<HttpResponse, BalanceError> {
    let balance_request = body.get_balance_request();
    let data = BalanceService::get_wallet_balance(
        pool.get_ref(),
        provider.get_ref(),
        &balance_request.get_chain(),
        &balance_request.get_currency(),
        user.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(data)))
}

pub async fn list_transactions(
    pool: Data<Pool<Postgres>>,
    query: Query<ListTransactionsParams>,
    user: ReqData<User>,
) -> Result<HttpResponse, TransactionError> {
    let query_params = query.into_inner();
    let data = WalletService::list_transactions(
        pool.get_ref(),
        query_params.page_size.unwrap_or(10),
        query_params.id,
        user.into_inner(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(BaseResponse::init(data)))
}

pub async fn poll_transaction(
    pool: Data<Pool<Postgres>>,
    query: Query<PollTransactionParams>,
    user: ReqData<User>,
) -> Result<HttpResponse, TransactionError> {
    let transaction = TransferService::get_status(
        pool.get_ref(),
        query.transaction_id.clone(),
        user.into_inner(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(BaseResponse::init(transaction)))
}
