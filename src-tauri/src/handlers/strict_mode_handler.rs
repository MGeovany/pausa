use tauri::State;

use crate::state::AppState;
use crate::strict_mode::StrictModeState;

/// Activate strict mode
#[tauri::command]
pub async fn activate_strict_mode(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.activate()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Deactivate strict mode
#[tauri::command]
pub async fn deactivate_strict_mode(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.deactivate()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Get the current strict mode state
#[tauri::command]
pub async fn get_strict_mode_state(
    app_state: State<'_, AppState>,
) -> Result<StrictModeState, String> {
    let orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_ref() {
        Ok(orchestrator.get_state())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Show menu bar popover
#[tauri::command]
pub async fn show_menu_bar_popover(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.show_menu_bar_popover()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Hide menu bar popover
#[tauri::command]
pub async fn hide_menu_bar_popover(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.hide_menu_bar_popover()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Stop break transition countdown
#[tauri::command]
pub async fn stop_break_transition_countdown(app_state: State<'_, AppState>) -> Result<(), String> {
    let orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if orchestrator_guard.is_some() {
        // This will be handled by the frontend countdown logic
        // Backend just needs to acknowledge the request
        println!("⏸️ [StrictModeHandler] Break transition countdown stopped");
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Start break from transition window
#[tauri::command]
pub async fn start_break_from_transition(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.start_break_from_transition()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Emergency exit from strict mode
#[tauri::command]
pub async fn emergency_exit_strict_mode(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.emergency_exit()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Hide fullscreen break overlay and unlock system
#[tauri::command]
pub async fn hide_fullscreen_break_overlay(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.hide_fullscreen_break_overlay()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Register emergency hotkey combination
#[tauri::command]
pub async fn register_emergency_hotkey(
    combination: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.register_emergency_hotkey(combination)?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Unregister emergency hotkey
#[tauri::command]
pub async fn unregister_emergency_hotkey(app_state: State<'_, AppState>) -> Result<(), String> {
    let mut orchestrator_guard = app_state.strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.unregister_emergency_hotkey()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}
