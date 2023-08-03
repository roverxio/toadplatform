use actix_web::web::{Data, Json, Path, Query};

use crate::errors::ApiError;
use crate::helpers::respond_json;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::models::wallet::address_response::AddressResponse;
use crate::models::wallet::balance_request::BalanceRequest;
use crate::models::wallet::balance_response::BalanceResponse;
use crate::services::balance_service::BalanceService;
use crate::services::transfer_service::TransactionService;
use crate::services::wallet_service::WalletService;

pub async fn get_address(service: Data<WalletService>) -> Result<Json<BaseResponse<AddressResponse>>, ApiError> {
    let wallet_address = service.get_wallet_address()?;
    respond_json(wallet_address)
}

pub async fn get_balance(service: Data<BalanceService>, body: Query<BalanceRequest>, entity: Path<String>) -> Result<Json<BaseResponse<BalanceResponse>>, ApiError> {
    println!("entity: {}", entity.into_inner());
    let balance_request = body.get_balance_request();
    let data = service.get_wallet_balance(&balance_request.chain, &balance_request.currency)?;
    respond_json(data)
}

pub async fn transact(service: Data<TransactionService>, body: Json<TransferRequest>) -> Result<Json<BaseResponse<TransactionResponse>>, ApiError> {
    let data = service.transfer_funds(body.into_inner())?;
    respond_json(data)
}
