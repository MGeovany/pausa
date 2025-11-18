use tauri::State;
use tokio::sync::Mutex;

use crate::strict_mode::{StrictModeOrchestrator, StrictModeState};

/// Activate strict mode
#[tauri::command]
pub async fn activate_strict_mode(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let mut orchestrator_guard = strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.activate()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Deactivate strict mode
#[tauri::command]
pub async fn deactivate_strict_mode(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let mut orchestrator_guard = strict_mode_orchestrator.lock().await;

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
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<StrictModeState, String> {
    let orchestrator_guard = strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_ref() {
        Ok(orchestrator.get_state())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Show menu bar popover
#[tauri::command]
pub async fn show_menu_bar_popover(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let mut orchestrator_guard = strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.show_menu_bar_popover()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Hide menu bar popover
#[tauri::command]
pub async fn hide_menu_bar_popover(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let mut orchestrator_guard = strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.hide_menu_bar_popover()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Stop break transition countdown
#[tauri::command]
pub async fn stop_break_transition_countdown(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let orchestrator_guard = strict_mode_orchestrator.lock().await;

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
pub async fn start_break_from_transition(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let mut orchestrator_guard = strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.start_break_from_transition()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}

/// Emergency exit from strict mode
#[tauri::command]
pub async fn emergency_exit_strict_mode(
    strict_mode_orchestrator: State<'_, Mutex<Option<StrictModeOrchestrator>>>,
) -> Result<(), String> {
    let mut orchestrator_guard = strict_mode_orchestrator.lock().await;

    if let Some(orchestrator) = orchestrator_guard.as_mut() {
        orchestrator.emergency_exit()?;
        Ok(())
    } else {
        Err("StrictModeOrchestrator not initialized".to_string())
    }
}
