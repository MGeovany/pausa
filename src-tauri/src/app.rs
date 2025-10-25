use tauri::Manager;

use crate::handlers::auth_handler;
use crate::{config::AppConfig, state::AppState};

pub fn run() -> Result<(), String> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let cfg = AppConfig::from_env()?;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let state = AppState::init(app.handle(), cfg.clone())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_handler::login_with_google,
            auth_handler::read_tokens,
            auth_handler::logout,
            auth_handler::get_user_info
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
