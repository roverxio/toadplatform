use jwks_client::keyset::KeyStore;
use serde::Deserialize;

pub struct AuthService;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthData {
    pub name: Option<String>,
    pub email: Option<String>,
    pub verifier_id: Option<String>,
}

impl AuthService {
    pub fn decode_jwt(token: &str) -> Result<AuthData, String> {
        let key_store = KeyStore::new();
        let jwt = key_store.decode(token).unwrap();

        if jwt.expired().unwrap_or(false) {
            Err("Sorry, token expired".to_string())
        } else {
            match jwt.payload().into::<AuthData>() {
                Ok(data) => Ok(data),
                Err(_) => Err("Failed to decode token".to_string()),
            }
        }
    }
}
