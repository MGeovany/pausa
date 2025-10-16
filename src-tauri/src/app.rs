use tauri::{Manager};
use tauri_plugin_deep_link::DeepLinkExt;

use crate::{config::AppConfig, state::AppState};
use crate::handlers::auth_handler;
use crate::infra::deeplink::handle_deep_link;

pub fn run() -> Result<(), String> {
    let cfg = AppConfig::from_env()?;
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(move |app| {
            let state = AppState::init(app, cfg.clone())?;
            app.manage(state);
            
            // Handle deep links
            app.deep_link().on_deep_link(move |app, url| {
                if let Err(e) = handle_deep_link(app, url) {
                    let _ = app.emit("auth:error", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_handler::login_with_google,
            auth_handler::read_tokens,
            auth_handler::get_tokens_path
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
