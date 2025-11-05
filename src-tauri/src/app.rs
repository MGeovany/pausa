use std::sync::Mutex;
use tauri::Manager;

use crate::handlers::{auth_handler, onboarding_handler};
use crate::{config::AppConfig, onboarding::OnboardingManager, state::AppState};

pub fn run() -> Result<(), String> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let cfg = AppConfig::from_env()?;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let state = AppState::init(app.handle(), cfg.clone())?;
            app.manage(state);

            // Initialize onboarding manager
            let onboarding_manager = OnboardingManager::new();
            app.manage(Mutex::new(onboarding_manager));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_handler::login_with_google,
            auth_handler::read_tokens,
            auth_handler::logout,
            auth_handler::get_user_info,
            onboarding_handler::start_onboarding,
            onboarding_handler::next_onboarding_step,
            onboarding_handler::previous_onboarding_step,
            onboarding_handler::complete_onboarding,
            onboarding_handler::get_onboarding_status
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
