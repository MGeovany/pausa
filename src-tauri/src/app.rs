use tauri::Manager;

use crate::{config::AppConfig, state::AppState};
use crate::handlers::auth_handler;

pub fn run() -> Result<(), String> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    let cfg = AppConfig::from_env()?;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(move |app| {
            let state = AppState::init(app.handle(), cfg.clone())?;
            app.manage(state);
            
            // TODO: Handle deep links - need to implement custom handler
            // The deep link plugin doesn't expose on_deep_link method directly
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_handler::login_with_google,
            auth_handler::read_tokens,
            auth_handler::get_tokens_path,
            auth_handler::logout,
            auth_handler::get_user_info
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
