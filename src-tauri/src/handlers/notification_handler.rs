use crate::state::AppState;
use tauri::State;

/// Update the user name for personalized notifications
#[tauri::command]
pub async fn update_notification_user_name(
    user_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "👤 [Rust] update_notification_user_name called: {:?}",
        user_name
    );

    let mut notification_service = state.notification_service.lock().await;
    notification_service.set_user_name(user_name);

    println!("✅ [Rust] Notification user name updated");

    Ok(())
}

/// Get the current user name used for notifications
#[tauri::command]
pub async fn get_notification_user_name(
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let notification_service = state.notification_service.lock().await;
    Ok(notification_service.get_user_name().map(|s| s.to_string()))
}
