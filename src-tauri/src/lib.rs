use tauri::Manager;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tokio::sync::mpsc;

mod database;
mod api_models;
mod state_manager;

use database::DatabaseManager;
use state_manager::{StateManager, StateEvent};
pub use api_models::*;

// Placeholder command - will be replaced in later tasks
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Database test command
#[tauri::command]
async fn get_database_stats(
    database: tauri::State<'_, Arc<Mutex<DatabaseManager>>>
) -> Result<String, String> {
    let db = database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    match db.get_stats() {
        Ok(stats) => Ok(format!(
            "Database Stats - Total Size: {} bytes, Used: {} bytes ({}%), Pages: {}",
            stats.total_size,
            stats.used_size(),
            stats.usage_percentage(),
            stats.page_count
        )),
        Err(e) => Err(format!("Failed to get database stats: {}", e))
    }
}

// State Manager Commands

#[tauri::command]
async fn start_focus_session(
    strict: bool,
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<FocusSession, String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    let events = manager.start_focus_session(strict)
        .map_err(|e| format!("Failed to start focus session: {}", e))?;
    
    // TODO: Emit events to frontend
    for event in events {
        println!("State Event: {:?}", event);
    }
    
    manager.get_current_session()
        .ok_or_else(|| "Failed to get current session after starting".to_string())
}

#[tauri::command]
async fn pause_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<(), String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    let events = manager.pause_session()
        .map_err(|e| format!("Failed to pause session: {}", e))?;
    
    // TODO: Emit events to frontend
    for event in events {
        println!("State Event: {:?}", event);
    }
    
    Ok(())
}

#[tauri::command]
async fn resume_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<(), String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    let events = manager.resume_session()
        .map_err(|e| format!("Failed to resume session: {}", e))?;
    
    // TODO: Emit events to frontend
    for event in events {
        println!("State Event: {:?}", event);
    }
    
    Ok(())
}

#[tauri::command]
async fn end_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<(), String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    let events = manager.end_session()
        .map_err(|e| format!("Failed to end session: {}", e))?;
    
    // TODO: Emit events to frontend
    for event in events {
        println!("State Event: {:?}", event);
    }
    
    Ok(())
}

#[tauri::command]
async fn get_current_session(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<Option<FocusSession>, String> {
    let manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(manager.get_current_session())
}

#[tauri::command]
async fn get_current_break(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<Option<BreakSession>, String> {
    let manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(manager.get_current_break())
}

#[tauri::command]
async fn complete_break(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<(), String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    let events = manager.complete_break()
        .map_err(|e| format!("Failed to complete break: {}", e))?;
    
    // TODO: Emit events to frontend
    for event in events {
        println!("State Event: {:?}", event);
    }
    
    Ok(())
}

#[tauri::command]
async fn get_app_state(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<String, String> {
    let manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(format!("{:?}", manager.get_state()))
}

#[tauri::command]
async fn update_settings(
    settings: UserSettings,
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<(), String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    manager.update_settings(settings)
        .map_err(|e| format!("Failed to update settings: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_settings(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<UserSettings, String> {
    let manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    Ok(manager.get_settings())
}

// Session statistics command
#[tauri::command]
async fn get_session_stats(
    days: u32,
    database: tauri::State<'_, Arc<Mutex<DatabaseManager>>>
) -> Result<Vec<SessionStats>, String> {
    let db = database.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    let db_stats = db.get_session_stats(days)
        .map_err(|e| format!("Failed to get session stats: {}", e))?;
    
    // Convert database stats to API stats
    let api_stats: Vec<SessionStats> = db_stats.into_iter().map(|s| s.into()).collect();
    
    Ok(api_stats)
}

// Test command to verify state manager functionality
#[tauri::command]
async fn test_state_manager(
    state_manager: tauri::State<'_, Arc<Mutex<StateManager>>>
) -> Result<String, String> {
    let mut manager = state_manager.lock().map_err(|e| format!("Failed to lock state manager: {}", e))?;
    
    // Test basic state
    let initial_state = manager.get_state();
    
    // Test starting a session
    let events = manager.start_focus_session(false)
        .map_err(|e| format!("Failed to start session: {}", e))?;
    
    let session_state = manager.get_state();
    let current_session = manager.get_current_session();
    
    // Test pausing
    let pause_events = manager.pause_session()
        .map_err(|e| format!("Failed to pause session: {}", e))?;
    
    // Test resuming
    let resume_events = manager.resume_session()
        .map_err(|e| format!("Failed to resume session: {}", e))?;
    
    // End the session
    let end_events = manager.end_session()
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            let db_path = app_data_dir.join("pausa.db");
            
            let database_manager = DatabaseManager::new(db_path)
                .expect("Failed to initialize database");
            
            // Create shared database reference
            let db_arc = Arc::new(Mutex::new(database_manager));
            
            // Initialize state manager
            let state_manager = StateManager::new(Arc::clone(&db_arc))
                .expect("Failed to initialize state manager");
            
            // Create shared state manager reference
            let state_manager_arc = Arc::new(Mutex::new(state_manager));
            
            // Start timer service
            let _timer_rx = StateManager::start_timer_service(Arc::clone(&state_manager_arc));
            // TODO: Handle timer events and emit to frontend
            
            // Store managers in app state
            app.manage(db_arc);
            app.manage(state_manager_arc);
            
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
            test_state_manager
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    fn create_test_managers() -> (Arc<Mutex<DatabaseManager>>, Arc<Mutex<StateManager>>) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_manager = DatabaseManager::new(db_path).unwrap();
        let db_arc = Arc::new(Mutex::new(database_manager));
        let state_manager = StateManager::new(Arc::clone(&db_arc)).unwrap();
        let state_manager_arc = Arc::new(Mutex::new(state_manager));
        (db_arc, state_manager_arc)
    }

    #[tokio::test]
    async fn test_start_focus_session_command() {
        let (_db_arc, state_manager_arc) = create_test_managers();
        
        // Test starting a focus session
        let result = start_focus_session(false, tauri::State::from(&state_manager_arc)).await;
        assert!(result.is_ok());
        
        let session = result.unwrap();
        assert_eq!(session.duration, 25 * 60); // 25 minutes default
        assert!(!session.is_strict);
        assert!(session.is_running);
    }

    #[tokio::test]
    async fn test_pause_resume_session_commands() {
        let (_db_arc, state_manager_arc) = create_test_managers();
        
        // Start a session first
        start_focus_session(false, tauri::State::from(&state_manager_arc)).await.unwrap();
        
        // Test pausing
        let pause_result = pause_session(tauri::State::from(&state_manager_arc)).await;
        assert!(pause_result.is_ok());
        
        // Test resuming
        let resume_result = resume_session(tauri::State::from(&state_manager_arc)).await;
        assert!(resume_result.is_ok());
    }

    #[tokio::test]
    async fn test_get_current_session_command() {
        let (_db_arc, state_manager_arc) = create_test_managers();
        
        // Initially no session
        let result = get_current_session(tauri::State::from(&state_manager_arc)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        
        // Start a session
        start_focus_session(true, tauri::State::from(&state_manager_arc)).await.unwrap();
        
        // Now should have a session
        let result = get_current_session(tauri::State::from(&state_manager_arc)).await;
        assert!(result.is_ok());
        let session = result.unwrap();
        assert!(session.is_some());
        assert!(session.unwrap().is_strict);
    }

    #[tokio::test]
    async fn test_settings_commands() {
        let (_db_arc, state_manager_arc) = create_test_managers();
        
        // Test getting default settings
        let result = get_settings(tauri::State::from(&state_manager_arc)).await;
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.focus_duration, 25);
        assert_eq!(settings.short_break_duration, 5);
        
        // Test updating settings
        let mut new_settings = settings;
        new_settings.focus_duration = 30;
        new_settings.strict_mode = true;
        
        let update_result = update_settings(new_settings.clone(), tauri::State::from(&state_manager_arc)).await;
        assert!(update_result.is_ok());
        
        // Verify settings were updated
        let result = get_settings(tauri::State::from(&state_manager_arc)).await;
        assert!(result.is_ok());
        let updated_settings = result.unwrap();
        assert_eq!(updated_settings.focus_duration, 30);
        assert!(updated_settings.strict_mode);
    }

    #[tokio::test]
    async fn test_session_stats_command() {
        let (db_arc, _state_manager_arc) = create_test_managers();
        
        // Test getting stats (should be empty initially)
        let result = get_session_stats(7, tauri::State::from(&db_arc)).await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert!(stats.is_empty() || stats.iter().all(|s| s.focus_minutes == 0));
    }

    #[tokio::test]
    async fn test_end_session_command() {
        let (_db_arc, state_manager_arc) = create_test_managers();
        
        // Start a session first
        start_focus_session(false, tauri::State::from(&state_manager_arc)).await.unwrap();
        
        // Test ending the session
        let result = end_session(tauri::State::from(&state_manager_arc)).await;
        assert!(result.is_ok());
        
        // Verify no current session
        let current = get_current_session(tauri::State::from(&state_manager_arc)).await.unwrap();
        assert!(current.is_none());
    }
}
