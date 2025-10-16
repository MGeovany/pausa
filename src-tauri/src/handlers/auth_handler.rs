use tauri::{AppHandle, State, Manager};
use crate::{state::AppState, domain::oauth::OAuthProvider};

#[tauri::command]
pub async fn login_with_google(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let mut svc = state.oauth_google.lock().map_err(|e| e.to_string())?;
    let url = svc.start_login().await.map_err(|e| e.to_string())?;
    tauri_plugin_opener::open(&app.shell(), &url, None::<&str>).map_err(|e| e.to_string())?;
    Ok("browser_opened".into())
}

#[tauri::command]
pub async fn read_tokens(state: State<'_, AppState>) -> Result<Option<crate::domain::tokens::Tokens>, String> {
    state.tokens_storage.load()
}

#[tauri::command]
pub async fn get_tokens_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.tokens_storage.path_str())
}
