use std::sync::Mutex;
use tauri::Manager;

use crate::handlers::{
    auth_handler, cycle_config_handler, cycle_handler, notification_handler, onboarding_handler,
    stats_handler, strict_mode_handler, work_schedule_handler,
};
use crate::{config::AppConfig, onboarding::OnboardingManager, state::AppState};

pub fn run() -> Result<(), String> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let cfg = AppConfig::from_env()?;
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let state = AppState::init(app.handle(), cfg.clone())?;
            app.manage(state);

            // Initialize onboarding manager
            let onboarding_manager = OnboardingManager::new();
            app.manage(Mutex::new(onboarding_manager));

            // Setup tray icon click handler
            if let Some(tray) = app.tray_by_id("main-tray") {
                tray.on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        // Get the app handle
                        let app_handle = tray.app_handle();

                        // Try to show menu bar popover
                        if let Some(window_manager) =
                            app_handle.try_state::<std::sync::Arc<
                                std::sync::Mutex<crate::window_manager::WindowManager>,
                            >>()
                        {
                            if let Ok(manager) = window_manager.lock() {
                                if let Err(e) = manager.show_menu_bar_popover() {
                                    eprintln!("Failed to show menu bar popover: {}", e);
                                }
                            }
                        }
                    }
                });
            }

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
            onboarding_handler::get_onboarding_status,
            onboarding_handler::is_first_launch,
            onboarding_handler::reset_onboarding_for_testing,
            onboarding_handler::apply_onboarding_config_to_settings,
            onboarding_handler::validate_onboarding_config,
            onboarding_handler::validate_step_config,
            onboarding_handler::create_configuration_backup,
            onboarding_handler::list_configuration_backups,
            onboarding_handler::restore_configuration_backup,
            onboarding_handler::get_configuration_health_check,
            onboarding_handler::force_database_migration,
            work_schedule_handler::save_work_schedule,
            work_schedule_handler::get_work_schedule,
            work_schedule_handler::is_within_work_hours,
            work_schedule_handler::get_system_timezone_info,
            work_schedule_handler::validate_work_hours,
            cycle_config_handler::save_cycle_config,
            cycle_config_handler::get_cycle_config,
            cycle_config_handler::get_user_settings,
            cycle_config_handler::update_user_name,
            cycle_config_handler::save_strict_mode_config,
            cycle_config_handler::get_strict_mode_config,
            cycle_config_handler::update_pre_alert_config,
            cycle_config_handler::get_pre_alert_config,
            cycle_config_handler::get_settings,
            cycle_config_handler::update_settings,
            cycle_handler::initialize_cycle_orchestrator,
            cycle_handler::start_focus_session,
            cycle_handler::start_break_session,
            cycle_handler::pause_cycle,
            cycle_handler::resume_cycle,
            cycle_handler::end_cycle_session,
            cycle_handler::get_cycle_state,
            cycle_handler::cycle_tick,
            cycle_handler::reset_cycle_count,
            cycle_handler::log_bypass_attempt,
            cycle_handler::get_work_schedule_info,
            cycle_handler::get_work_hours_stats,
            stats_handler::get_session_stats,
            notification_handler::update_notification_user_name,
            notification_handler::get_notification_user_name,
            strict_mode_handler::activate_strict_mode,
            strict_mode_handler::deactivate_strict_mode,
            strict_mode_handler::get_strict_mode_state,
            strict_mode_handler::show_menu_bar_popover,
            strict_mode_handler::hide_menu_bar_popover,
            strict_mode_handler::stop_break_transition_countdown,
            strict_mode_handler::start_break_from_transition,
            strict_mode_handler::hide_fullscreen_break_overlay,
            strict_mode_handler::emergency_exit_strict_mode
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
