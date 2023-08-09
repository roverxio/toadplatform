use std::error::Error;
use ethers::types::Bytes;
use reqwest;
use crate::CONFIG;
use crate::models::contract_interaction::user_operation::UserOperation;
use crate::models::signing::signed_message::SignedPayload;
use crate::models::signing::signing_payload::SigningPayload;

#[derive(Clone)]
pub struct HttpClient {
    pub client: reqwest::Client,
}

impl HttpClient {
    pub async fn sign_message(
        &self,
        user_operation: UserOperation,
        entry_point: String,
        chain_id: u64,
    ) -> Result<Bytes, Box<dyn Error>> {

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);

        let data = SigningPayload {
            user_operation,
            entrypoint_address: entry_point,
            chain_id,
        };

        println!("Signing payload: {:?}", data);

        let response = self.client
            .post(CONFIG.urls.signing_server.clone())
            .headers(headers)
            .json(&data)
            .send()
            .await?;

        if response.status().is_success() {
            let body: SignedPayload = response.json().await?;
            Ok(body.sign)
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch the sign value",
            )))
        }
    }
}
