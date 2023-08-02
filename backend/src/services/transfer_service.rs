use crate::errors::ApiError;
use crate::models::transfer::transfer_request::TransferRequest;
use crate::models::transfer::transfer_response::TransactionResponse;

#[derive(Clone)]
pub struct TransactionService {}

impl TransactionService {
    pub fn transfer_funds(&self, request: TransferRequest) -> Result<TransactionResponse, ApiError> {
        println!("Transferring funds: {:?}", request.value);
        Ok(TransactionResponse {
            transaction_hash: "hash".to_string(),
            status: "success".to_string(),
            explorer: "no".to_string(),
        })
    }
}
