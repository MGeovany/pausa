use std::path::PathBuf;
use tauri::Manager;

#[derive(Clone)]
pub struct AppConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| "GOOGLE_CLIENT_ID no definido".to_string())?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok();
        let redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8080".into());
        Ok(Self { client_id, client_secret, redirect_uri })
    }
}

pub fn tokens_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir()
        .map_err(|e| format!("no app data dir: {}", e))?;
    Ok(dir.join("auth").join("google_tokens.json"))
}
