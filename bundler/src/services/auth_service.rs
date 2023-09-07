use jwks_client::keyset::KeyStore;
use rs_firebase_admin_sdk::{
    auth::{FirebaseAuthService, UserIdentifiers},
    App, GcpCredentials,
};
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

    pub async fn is_valid_id(verifier_id: String) -> bool {
        let live_app = App::live(GcpCredentials::new().await.unwrap())
            .await
            .unwrap();

        let auth_admin = live_app.auth();

        let user = auth_admin
            .get_user(
                // Build a filter for finding the user
                UserIdentifiers::builder().with_uid(verifier_id).build(),
            )
            .await
            .expect("Error while fetching user");
        user.is_some()
    }
}
