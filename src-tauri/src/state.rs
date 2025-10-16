use std::sync::Mutex;
use tauri::{AppHandle};

use crate::{config::{AppConfig, tokens_path}};
use crate::domain::tokens::TokenStorage;
use crate::services::google_oauth::GoogleOAuthService;

pub struct AppState {
    pub oauth_google: Mutex<GoogleOAuthService>,
    pub tokens_storage: TokenStorage,
}

impl AppState {
    pub fn init(app: &AppHandle, cfg: AppConfig) -> Result<Self, String> {
        let tokens_path = tokens_path(app)?;
        let storage = TokenStorage::new(tokens_path);
        let svc = GoogleOAuthService::new(cfg);
        Ok(Self {
            oauth_google: Mutex::new(svc),
            tokens_storage: storage,
        })
    }
}
