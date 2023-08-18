use actix_web::web::{Data, Json, Query};
use actix_web::HttpRequest;

use crate::errors::ApiError;
use crate::models::response::base_response::BaseResponse;
use crate::models::transaction::list_transactions::ListTransactions;
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
            &balance_request.chain,
            &balance_request.currency.to_lowercase(),
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
    let data = service
        .transfer_funds(body.into_inner(), &get_user(req))
        .await?;
    respond_json(data)
}

pub async fn list_transactions(
    service: Data<WalletService>,
    query: Query<ListTransactions>,
) -> Result<Json<BaseResponse<Vec<Transaction>>>, ApiError> {
    let query_params = query.into_inner();
    let data = service.list_transactions(query_params.page_size, query_params.id);
    respond_json(data)
}
