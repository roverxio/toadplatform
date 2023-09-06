use actix_web::error::ErrorUnauthorized;
use actix_web::web::{Data, Json, Query};
use actix_web::{Error, HttpRequest, HttpResponse};
use sqlx::{Pool, Postgres};

use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
use crate::models::transaction::list_transactions_params::ListTransactionsParams;
use crate::models::transaction::poll_transaction_params::PollTransactionParams;
use crate::models::transaction::transaction::Transaction;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransferResponse;
use crate::models::wallet::address_response::AddressResponse;
use crate::models::wallet::balance_request::BalanceRequest;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::provider::helpers::{get_user, respond_json};
use crate::services::balance_service::BalanceService;
use crate::services::transfer_service::TransferService;
use crate::services::wallet_service::WalletService;

pub async fn get_address(
    service: Data<WalletService>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<AddressResponse>>, ApiError> {
    let wallet_address = service.get_wallet_address(&get_user(req)).await?;
    respond_json(wallet_address)
}

pub async fn get_balance(
    service: Data<BalanceService>,
    body: Query<BalanceRequest>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<BalanceResponse>>, ApiError> {
    let balance_request = body.get_balance_request();
    let data = service
        .get_wallet_balance(
            &balance_request.get_chain(),
            &balance_request.get_currency(),
            &get_user(req),
        )
        .await?;
    respond_json(data)
}

pub async fn transfer(
    service: Data<TransferService>,
    body: Json<TransferRequest>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<TransferResponse>>, ApiError> {
    let body = body.into_inner();
    let data = service
        .transfer_funds(
            body.get_receiver(),
            body.get_value(),
            body.metadata.get_currency(),
            &get_user(req),
        )
        .await?;
    respond_json(data)
}

pub async fn list_transactions(
    service: Data<WalletService>,
    query: Query<ListTransactionsParams>,
    req: HttpRequest,
) -> Result<Json<BaseResponse<Vec<Transaction>>>, ApiError> {
    let query_params = query.into_inner();
    let data = service
        .list_transactions(
            query_params.page_size.unwrap_or(10),
            query_params.id,
            &get_user(req),
        )
        .await;
    respond_json(data)
}

pub async fn poll_transaction(
    db_pool: Data<Pool<Postgres>>,
    query: Query<PollTransactionParams>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let user_id = get_user(req);
    if user_id.is_empty() {
        return Err(ErrorUnauthorized(""));
    }

    let transaction =
        TransferService::get_status(db_pool.get_ref(), query.transaction_id.clone(), user_id)
            .await
            .unwrap();

    Ok(HttpResponse::Ok().json(BaseResponse {
        data: transaction,
        err: Default::default(),
    }))
}
