use tauri::State;
use crate::state::AppState;
use crate::services::telemetry::{ErrorEvent, LoginEvent, MetricEvent};

#[tauri::command]
pub async fn send_error_event(
    error_type: String,
    message: String,
    context: Option<String>,
    stack: Option<String>,
    user_action: Option<String>,
    recoverable: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let error = ErrorEvent {
        error_type,
        message,
        context,
        stack,
        user_action,
        recoverable,
    };
    
    state.telemetry_service.log_error(error).await;
    
    // Auto-flush errors immediately
    state.telemetry_service.flush().await
        .map_err(|e| format!("Failed to send error event: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn send_login_event(
    event: String,
    provider: String,
    error: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let login_event = LoginEvent {
        event,
        provider,
        error,
    };
    
    state.telemetry_service.log_login(login_event).await;
    state.telemetry_service.flush().await
        .map_err(|e| format!("Failed to send login event: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn send_metric(
    metric_name: String,
    value: f64,
    tags: Option<serde_json::Value>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let metric = MetricEvent {
        metric_name,
        value,
        tags,
    };
    
    state.telemetry_service.log_metric(metric).await;
    
    Ok(())
}

#[tauri::command]
pub async fn flush_telemetry(state: State<'_, AppState>) -> Result<(), String> {
    state.telemetry_service.flush().await
        .map_err(|e| format!("Failed to flush telemetry: {}", e))
}
