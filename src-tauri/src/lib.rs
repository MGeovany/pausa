use tauri::Manager;
use std::sync::Mutex;
use std::path::PathBuf;

mod database;
mod api_models;

use database::DatabaseManager;
pub use api_models::*;

// Placeholder command - will be replaced in later tasks
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Database test command
#[tauri::command]
async fn get_database_stats(
    database: tauri::State<'_, Mutex<DatabaseManager>>
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
            
            // Store database manager in app state
            app.manage(Mutex::new(database_manager));
            
            // Hide the main window on startup - we'll manage windows manually
            let main_window = app.get_webview_window("main").unwrap();
            main_window.hide().unwrap();
            
            println!("Pausa application initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, get_database_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
