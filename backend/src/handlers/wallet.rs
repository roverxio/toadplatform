use actix_web::web::{Data, Json};

use crate::errors::ApiError;
use crate::helpers::respond_json;
use crate::models::response::base_response::BaseResponse;
use crate::models::wallet::address_response::AddressResponse;
use crate::services::wallet_service::WalletService;

pub async fn get_address(service: Data<WalletService>) -> Result<Json<BaseResponse<AddressResponse>>, ApiError> {
    let wallet_address = service.get_wallet_address()?;
    respond_json(wallet_address)
}
