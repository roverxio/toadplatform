use actix_web::web::{Data, Json};

use crate::errors::ApiError;
use crate::helpers::respond_json;
use crate::models::admin::paymaster_topup::PaymasterTopup;
use crate::models::response::base_response::BaseResponse;
use crate::models::transfer::transfer_response::TransactionResponse;
use crate::services::admin_service::AdminService;

pub async fn topup_paymaster_deposit(service: Data<AdminService>, body: Json<PaymasterTopup>) -> Result<Json<BaseResponse<TransactionResponse>>, ApiError> {
    let hello = service.topup_paymaster_deposit(body.into_inner())?;
    respond_json(hello)
}
