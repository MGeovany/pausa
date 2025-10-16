use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: i64,
    pub scope: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub obtained_at: i64,
}

impl Tokens {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        now >= self.obtained_at + self.expires_in - 30 // margen 30s
    }
}

pub struct TokenStorage { path: PathBuf }

impl TokenStorage {
    pub fn new(path: PathBuf) -> Self { Self { path } }
    pub fn path_str(&self) -> String { self.path.to_string_lossy().to_string() }

    pub fn load(&self) -> Result<Option<Tokens>, String> {
        match fs::read_to_string(&self.path) {
            Ok(s) => Ok(Some(serde_json::from_str(&s).map_err(|e| e.to_string())?)),
            Err(_) => Ok(None),
        }
    }
    pub fn save(&self, t: &Tokens) -> Result<(), String> {
        if let Some(dir) = self.path.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
        let s = serde_json::to_string_pretty(t).map_err(|e| e.to_string())?;
        fs::write(&self.path, s).map_err(|e| e.to_string())
    }
}
