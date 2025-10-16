use std::sync::{Arc, Mutex};
use tauri::Manager;

mod api_models;
mod database;
mod hotkey_manager;
mod state_manager;
mod window_manager;

pub use api_models::*;
use database::DatabaseManager;
use hotkey_manager::HotkeyManager;
use state_manager::{StateEvent, StateManager};
use window_manager::WindowManager;

// Helper function to emit state events to frontend
fn emit_state_event(app_handle: &tauri::AppHandle, event: &StateEvent) {
    match event {
        StateEvent::StateChanged { from, to } => {
            let payload = serde_json::json!({
                "from": format!("{:?}", from),
                "to": format!("{:?}", to)
            });
            if let Err(e) = app_handle.emit("state-change", payload) {
                eprintln!("Failed to emit state-change event: {}", e);
            }
        }
        StateEvent::SessionStarted { session_id } => {
            if let Err(e) = app_handle.emit("session-started", session_id) {
                eprintln!("Failed to emit session-started event: {}", e);
            }
        }
        StateEvent::SessionPaused { session_id } => {
            if let Err(e) = app_handle.emit("session-paused", session_id) {
                eprintln!("Failed to emit session-paused event: {}", e);
            }
        }
        StateEvent::SessionResumed { session_id } => {
            if let Err(e) = app_handle.emit("session-resumed", session_id) {
                eprintln!("Failed to emit session-resumed event: {}", e);
            }
        }
        StateEvent::SessionCompleted { session_id } => {
            if let Err(e) = app_handle.emit("session-completed", session_id) {
                eprintln!("Failed to emit session-completed event: {}", e);
            }
        }
        StateEvent::PreAlertTriggered {
            session_id,
            remaining_seconds,
        } => {
            let payload = serde_json::json!({
                "sessionId": session_id,
                "remainingSeconds": remaining_seconds
            });
            if let Err(e) = app_handle.emit("pre-alert-triggered", payload) {
                eprintln!("Failed to emit pre-alert-triggered event: {}", e);
            }
        }
        StateEvent::BreakStarted { break_session } => {
            if let Err(e) = app_handle.emit("break-update", break_session) {
                eprintln!("Failed to emit break-update event: {}", e);
            }
        }
        StateEvent::BreakCompleted { session_id } => {
            if let Err(e) = app_handle.emit("break-completed", session_id) {
                eprintln!("Failed to emit break-completed event: {}", e);
            }
        }
        StateEvent::TimerTick {
            remaining_seconds: _,
        } => {
            // Timer ticks are handled by emitting session updates, not separate events
        }
    }
}

// Placeholder command - will be replaced in later tasks
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Database test command
#[tauri::command]
async fn get_database_stats(
    database: tauri::State<'_, Arc<Mutex<DatabaseManager>>>,
) -> Result<String, String> {
    let db = database
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    match db.get_stats() {
        Ok(stats) => Ok(format!(
            "Database Stats - Total Size: {} bytes, Used: {} bytes ({}%), Pages: {}",
            stats.total_size,
            stats.used_size(),
            stats.usage_percentage(),
            stats.page_count
        )),
        Err(e) => Err(format!("Failed to get database stats: {}", e)),
    }
}

// State Manager Commands

#[tauri::command]
async fn start_focus_session(
    strict: bool,
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
    app_handle: tauri::AppHandle,
) -> Result<FocusSession, String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    let events = manager
        .start_focus_session(strict)
        .map_err(|e| format!("Failed to start focus session: {}", e))?;

    let session = manager
        .get_current_session()
        .ok_or_else(|| "Failed to get current session after starting".to_string())?;

    // Emit events to frontend
    for event in events {
        emit_state_event(&app_handle, &event);
    }

    // Emit session update
    if let Err(e) = app_handle.emit("session-update", &session) {
        eprintln!("Failed to emit session-update event: {}", e);
    }

    Ok(session)
}

#[tauri::command]
async fn pause_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    let events = manager
        .pause_session()
        .map_err(|e| format!("Failed to pause session: {}", e))?;

    // Emit events to frontend
    for event in events {
        emit_state_event(&app_handle, &event);
    }

    // Emit updated session
    if let Some(session) = manager.get_current_session() {
        if let Err(e) = app_handle.emit("session-update", &session) {
            eprintln!("Failed to emit session-update event: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
async fn resume_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    let events = manager
        .resume_session()
        .map_err(|e| format!("Failed to resume session: {}", e))?;

    // Emit events to frontend
    for event in events {
        emit_state_event(&app_handle, &event);
    }

    // Emit updated session
    if let Some(session) = manager.get_current_session() {
        if let Err(e) = app_handle.emit("session-update", &session) {
            eprintln!("Failed to emit session-update event: {}", e);
        }
    }

    Ok(())
}

#[tauri::command]
async fn end_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    let events = manager
        .end_session()
        .map_err(|e| format!("Failed to end session: {}", e))?;

    // Emit events to frontend
    for event in events {
        emit_state_event(&app_handle, &event);
    }

    // Emit session cleared (null)
    if let Err(e) = app_handle.emit("session-update", Option::<FocusSession>::None) {
        eprintln!("Failed to emit session-update event: {}", e);
    }

    Ok(())
}

#[tauri::command]
async fn get_current_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
) -> Result<Option<FocusSession>, String> {
    let manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(manager.get_current_session())
}

#[tauri::command]
async fn get_current_break(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
) -> Result<Option<BreakSession>, String> {
    let manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(manager.get_current_break())
}

#[tauri::command]
async fn complete_break(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    let events = manager
        .complete_break()
        .map_err(|e| format!("Failed to complete break: {}", e))?;

    // Emit events to frontend
    for event in events {
        emit_state_event(&app_handle, &event);
    }

    // Emit break cleared (null)
    if let Err(e) = app_handle.emit("break-update", Option::<BreakSession>::None) {
        eprintln!("Failed to emit break-update event: {}", e);
    }

    Ok(())
}

#[tauri::command]
async fn get_app_state(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
) -> Result<String, String> {
    let manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(format!("{:?}", manager.get_state()))
}

#[tauri::command]
async fn update_settings(
    settings: UserSettings,
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
) -> Result<(), String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    manager
        .update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_settings(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
) -> Result<UserSettings, String> {
    let manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(manager.get_settings())
}

// Session statistics command
#[tauri::command]
async fn get_session_stats(
    days: u32,
    database: tauri::State<'_, Arc<Mutex<DatabaseManager>>>,
) -> Result<Vec<SessionStats>, String> {
    let db = database
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    let db_stats = db
        .get_session_stats(days)
        .map_err(|e| format!("Failed to get session stats: {}", e))?;

    // Convert database stats to API stats
    let api_stats: Vec<SessionStats> = db_stats.into_iter().map(|s| s.into()).collect();

    Ok(api_stats)
}

// Test command to verify state manager functionality
#[tauri::command]
async fn test_state_manager(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>,
) -> Result<String, String> {
    let mut manager = state_manager
        .lock()
        .map_err(|e| format!("Failed to lock state manager: {}", e))?;

    // Test basic state
    let initial_state = manager.get_state();

    // Test starting a session
    let events = manager
        .start_focus_session(false)
        .map_err(|e| format!("Failed to start session: {}", e))?;

    let session_state = manager.get_state();
    let current_session = manager.get_current_session();

    // Test pausing
    let pause_events = manager
        .pause_session()
        .map_err(|e| format!("Failed to pause session: {}", e))?;

    // Test resuming
    let resume_events = manager
        .resume_session()
        .map_err(|e| format!("Failed to resume session: {}", e))?;

    // End the session
    let end_events = manager
        .end_session()
        .map_err(|e| format!("Failed to end session: {}", e))?;

    let final_state = manager.get_state();

    Ok(format!(
        "State Manager Test Results:\n\
         Initial State: {:?}\n\
         Session State: {:?}\n\
         Final State: {:?}\n\
         Session Created: {}\n\
         Events Generated: {} start, {} pause, {} resume, {} end",
        initial_state,
        session_state,
        final_state,
        current_session.is_some(),
        events.len(),
        pause_events.len(),
        resume_events.len(),
        end_events.len()
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            let db_path = app_data_dir.join("pausa.db");

            let database_manager =
                DatabaseManager::new(db_path).expect("Failed to initialize database");

            // Create shared database reference
            let db_arc = Arc::new(Mutex::new(database_manager));

            // Initialize state manager
            let state_manager =
                StateManager::new(Arc::clone(&db_arc)).expect("Failed to initialize state manager");

            // Create shared state manager reference
            let state_manager_arc = Arc::new(Mutex::new(state_manager));

            // Start timer service in a separate thread with its own Tokio runtime
            let state_manager_for_timer = Arc::clone(&state_manager_arc);
            let app_handle_for_timer = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
                rt.block_on(async {
                    let mut timer_rx =
                        StateManager::start_timer_service(state_manager_for_timer.clone());

                    // Handle timer events and emit to frontend
                    while let Some(events) = timer_rx.recv().await {
                        for event in events {
                            emit_state_event(&app_handle_for_timer, &event);
                        }

                        // Emit current session state after timer tick
                        if let Ok(manager) = state_manager_for_timer.try_lock() {
                            if let Some(session) = manager.get_current_session() {
                                if let Err(e) =
                                    app_handle_for_timer.emit("session-update", &session)
                                {
                                    eprintln!("Failed to emit session-update from timer: {}", e);
                                }
                            }
                            if let Some(break_session) = manager.get_current_break() {
                                if let Err(e) =
                                    app_handle_for_timer.emit("break-update", &break_session)
                                {
                                    eprintln!("Failed to emit break-update from timer: {}", e);
                                }
                            }
                        }
                    }
                });
            });

            // Initialize window manager
            let window_manager = WindowManager::new(app.handle().clone());
            let window_manager_arc = Arc::new(Mutex::new(window_manager));

            // Initialize hotkey manager
            let hotkey_manager = HotkeyManager::new(
                app.handle().clone(),
                Arc::clone(&state_manager_arc),
                Arc::clone(&window_manager_arc),
            );

            // Initialize hotkeys
            if let Err(e) = hotkey_manager.initialize() {
                eprintln!("Warning: Failed to initialize hotkeys: {}", e);
            }

            let hotkey_manager_arc = Arc::new(Mutex::new(hotkey_manager));

            // Store managers in app state
            app.manage(db_arc);
            app.manage(state_manager_arc);
            app.manage(window_manager_arc);
            app.manage(hotkey_manager_arc);

            println!("Pausa application initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_database_stats,
            start_focus_session,
            pause_session,
            resume_session,
            end_session,
            get_current_session,
            get_current_break,
            complete_break,
            get_app_state,
            update_settings,
            get_settings,
            get_session_stats,
            test_state_manager,
            window_manager::show_command_palette,
            window_manager::hide_command_palette,
            window_manager::toggle_command_palette,
            window_manager::show_focus_widget,
            window_manager::hide_focus_widget,
            window_manager::show_break_overlay,
            window_manager::hide_break_overlay,
            window_manager::show_settings,
            window_manager::hide_settings,
            window_manager::handle_focus_widget_drag,
            window_manager::is_window_visible,
            hotkey_manager::get_hotkey_configs,
            hotkey_manager::update_hotkey_config,
            hotkey_manager::set_hotkey_enabled,
            hotkey_manager::refresh_hotkey_state,
            hotkey_manager::route_hotkey_event,
            hotkey_manager::save_hotkey_configurations,
            hotkey_manager::load_custom_hotkeys,
            hotkey_manager::reset_hotkeys_to_defaults,
            hotkey_manager::check_hotkey_conflicts,
            hotkey_manager::get_available_modifiers
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
