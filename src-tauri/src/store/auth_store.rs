use serde::{Deserialize, Serialize};

use super::store_manager::StoreManager;

#[derive(Default, Serialize, Deserialize)]
pub struct AuthStore {
    auth_token: String,
}

impl StoreManager {
    pub fn set_auth_token(&self, token: String) {
        let mut auth_store = self.auth_store.lock();
        auth_store.auth_token = token;
        auth_store.update();
    }

    pub fn auth_token(&self) -> String {
        let auth_store = self.auth_store.lock();
        auth_store.auth_token.clone()
    }
}
