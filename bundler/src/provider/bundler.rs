use log::error;
use serde_json::Value;

use crate::errors::errors::ApiError;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::provider::lib::{EstimateResult, Request, Response};
use crate::CONFIG;

pub async fn estimate_gas(user_op0: UserOperation) -> Result<EstimateResult, ApiError> {
    let req_body = Request {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "eth_estimateUserOperationGas".to_string(),
        params: vec![
            serde_json::to_value(&user_op0).unwrap(),
            format!("{:?}", CONFIG.get_chain().entrypoint_address).into(),
        ],
    };
    let post = post(&req_body).await;
    let result =
        serde_json::from_str::<Response<EstimateResult>>(&post.text().await.unwrap()).unwrap();
    if result.error.is_some() {
        error!("could not estimate gas: {:?}", result.error);
        return Err(ApiError::InternalServer(
            "Failed to estimate gas".to_string(),
        ));
    }
    Ok(result.result.unwrap())
}

pub async fn submit_transaction(user_operation: UserOperation) -> Result<String, ApiError> {
    let send_body = Request {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "eth_sendUserOperation".to_string(),
        params: vec![
            serde_json::to_value(&user_operation).unwrap(),
            format!("{:?}", CONFIG.get_chain().entrypoint_address).into(),
        ],
    };
    let post = post(&send_body).await;

    let res = post.text().await.unwrap();
    let result = serde_json::from_str::<Response<String>>(&res).unwrap();

    if result.error.is_some() {
        return Err(ApiError::InternalServer(
            "Failed to submit transaction".to_string(),
        ));
    }
    Ok(result.result.unwrap())
}

async fn post(send_body: &Request<Vec<Value>>) -> reqwest::Response {
    let post = reqwest::Client::builder()
        .build()
        .unwrap()
        .post(CONFIG.get_chain().bundler_url.clone())
        .json(&send_body)
        .send()
        .await
        .unwrap();
    post
}
