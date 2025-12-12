use std::sync::Mutex;
use tauri::Manager;

use crate::handlers::{
    auth_handler, cycle_config_handler, cycle_handler, notification_handler, onboarding_handler,
    stats_handler, strict_mode_handler, work_schedule_handler,
};
use crate::{config::AppConfig, onboarding::OnboardingManager, state::AppState};

// Menu bar text temporarily disabled
// #[cfg(target_os = "macos")]
// use crate::menu_bar_text;

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

            // Initialize native menu bar text support on macOS
            // TEMPORARILY DISABLED: This was causing fatal runtime errors
            // The Objective-C code may be throwing exceptions that Rust cannot catch
            // TODO: Re-implement with proper exception handling or use Tauri's native APIs
            #[cfg(target_os = "macos")]
            {
                println!("⚠️ [App] Menu bar text initialization temporarily disabled to avoid fatal errors");
                // Disabled for now:
                // tauri::async_runtime::spawn(async move {
                //     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                //     match menu_bar_text::init_menu_bar_text() {
                //         Ok(_) => println!("✅ [App] Native menu bar text initialized"),
                //         Err(e) => eprintln!("⚠️ [App] Failed to initialize menu bar text: {}", e),
                //     }
                // });
            }

            // Setup tray icon with context menu (similar to Docker Desktop)
            println!("🔍 [App] Looking for tray icon with id 'main-tray'");
            if let Some(tray) = app.tray_by_id("main-tray") {
                println!("✅ [App] Tray icon found, setting up menu");
                use tauri::{menu::{Menu, MenuItem, PredefinedMenuItem}, tray::MouseButton};
                
                // Create context menu
                let app_handle = app.handle();
                
                let show_window = MenuItem::with_id(app_handle, "show-window", "Show Window", true, None::<&str>)?;
                let separator1 = PredefinedMenuItem::separator(app_handle)?;
                let toggle_strict_mode = MenuItem::with_id(app_handle, "toggle-strict-mode", "Toggle Strict Mode", true, None::<&str>)?;
                let view_stats = MenuItem::with_id(app_handle, "view-stats", "View Statistics", true, None::<&str>)?;
                let separator2 = PredefinedMenuItem::separator(app_handle)?;
                let settings = MenuItem::with_id(app_handle, "settings", "Settings...", true, None::<&str>)?;
                let separator3 = PredefinedMenuItem::separator(app_handle)?;
                let quit = PredefinedMenuItem::quit(app_handle, Some("Quit Pausa"))?;
                
                let menu = Menu::with_items(app_handle, &[
                    &show_window,
                    &separator1,
                    &toggle_strict_mode,
                    &view_stats,
                    &separator2,
                    &settings,
                    &separator3,
                    &quit,
                ])?;
                
                // Set the menu on the tray icon
                match tray.set_menu(Some(menu)) {
                    Ok(_) => println!("✅ [App] Menu set successfully on tray icon"),
                    Err(e) => {
                        eprintln!("❌ [App] Failed to set menu on tray icon: {}", e);
                        return Err(format!("Failed to set tray menu: {}", e).into());
                    }
                }
                
                // Handle menu item clicks
                let app_handle_clone = app.handle().clone();
                tray.on_menu_event(move |_tray, event| {
                    let app_handle = app_handle_clone.clone();
                    match event.id.as_ref() {
                        "show-window" => {
                            println!("🖱️ [TrayMenu] Show window clicked");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "toggle-strict-mode" => {
                            println!("🖱️ [TrayMenu] Toggle strict mode clicked - redirecting to settings");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                // Navigate to settings page where user can toggle strict mode
                                let _ = window.eval("window.location.hash = '#/settings';");
                            }
                        }
                        "view-stats" => {
                            println!("🖱️ [TrayMenu] View stats clicked");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                // Navigate to stats page
                                let _ = window.eval("window.location.hash = '#/stats';");
                            }
                        }
                        "settings" => {
                            println!("🖱️ [TrayMenu] Settings clicked");
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                // Navigate to settings page
                                let _ = window.eval("window.location.hash = '#/settings';");
                            }
                        }
                        "quit" => {
                            println!("🖱️ [TrayMenu] Quit clicked");
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                });
                
                // Also handle left click to show/hide window (macOS behavior)
                tray.on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        let app_handle = tray.app_handle().clone();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });
                println!("✅ [App] Tray icon event handlers registered");
            } else {
                eprintln!("❌ [App] Tray icon 'main-tray' not found! Check tauri.conf.json configuration.");
                eprintln!("⚠️ [App] Make sure trayIcon is configured in tauri.conf.json");
            }

            // Ensure the main window starts centered
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.center();
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
            cycle_handler::get_current_break,
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
            strict_mode_handler::emergency_exit_strict_mode,
            strict_mode_handler::register_emergency_hotkey,
            strict_mode_handler::unregister_emergency_hotkey
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.to_string())
}
