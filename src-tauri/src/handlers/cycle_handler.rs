use crate::cycle_orchestrator::{CycleConfig, CycleOrchestrator, CycleState};
use crate::database::models::{UserSettings, WorkSchedule};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeCycleRequest {
    pub force_reload: Option<bool>,
}

/// Initialize the cycle orchestrator with current user settings
#[tauri::command]
pub async fn initialize_cycle_orchestrator(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<CycleState, String> {
    println!("🔄 [Rust] initialize_cycle_orchestrator called");

    // Get user settings
    let user_settings = state
        .database
        .with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    r#"
                SELECT id, focus_duration, short_break_duration, long_break_duration,
                       cycles_per_long_break, cycles_per_long_break_v2, pre_alert_seconds,
                       strict_mode, pin_hash, user_name, emergency_key_combination,
                       created_at, updated_at
                FROM user_settings 
                WHERE id = 1
                "#,
                )
                .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

            let settings = stmt
                .query_row([], |row| UserSettings::from_row(row))
                .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

            Ok(settings)
        })
        .map_err(|e| format!("Failed to get user settings: {}", e))?;

    // Get work schedule
    let work_schedule = state
        .database
        .with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    r#"
                SELECT id, user_id, use_work_schedule, work_start_time, 
                       work_end_time, timezone, created_at, updated_at
                FROM work_schedule 
                WHERE id = 1
                "#,
                )
                .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

            let schedule = stmt.query_row([], |row| WorkSchedule::from_row(row)).ok();

            Ok(schedule)
        })
        .map_err(|e| format!("Failed to get work schedule: {}", e))?;

    // Create cycle config
    let config = CycleConfig::from_user_settings(user_settings, work_schedule);

    // Create orchestrator
    let orchestrator = CycleOrchestrator::new(config);

    let current_state = orchestrator.get_state();

    // Store in app state
    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;
    *cycle_orchestrator = Some(orchestrator);

    println!("✅ [Rust] Cycle orchestrator initialized");

    Ok(current_state)
}

/// Start a focus session
#[tauri::command]
pub async fn start_focus_session(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<CycleState, String> {
    println!("▶️ [Rust] start_focus_session called");

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let events = orchestrator.start_focus_session()?;

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    let current_state = orchestrator.get_state();

    println!("✅ [Rust] Focus session started");

    Ok(current_state)
}

/// Start a break (short or long)
#[tauri::command]
pub async fn start_break_session(
    force_long: Option<bool>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<CycleState, String> {
    println!(
        "☕ [Rust] start_break_session called (force_long: {:?})",
        force_long
    );

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let events = orchestrator.start_break(force_long.unwrap_or(false))?;

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    let current_state = orchestrator.get_state();

    println!("✅ [Rust] Break session started");

    Ok(current_state)
}

/// Pause the current session
#[tauri::command]
pub async fn pause_cycle(state: State<'_, AppState>) -> Result<CycleState, String> {
    println!("⏸️ [Rust] pause_cycle called");

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    orchestrator.pause()?;

    let current_state = orchestrator.get_state();

    println!("✅ [Rust] Cycle paused");

    Ok(current_state)
}

/// Resume the current session
#[tauri::command]
pub async fn resume_cycle(state: State<'_, AppState>) -> Result<CycleState, String> {
    println!("▶️ [Rust] resume_cycle called");

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    orchestrator.resume()?;

    let current_state = orchestrator.get_state();

    println!("✅ [Rust] Cycle resumed");

    Ok(current_state)
}

/// End the current session
#[tauri::command]
pub async fn end_cycle_session(
    completed: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<CycleState, String> {
    println!(
        "⏹️ [Rust] end_cycle_session called (completed: {})",
        completed
    );

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let events = orchestrator.end_session(completed)?;

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    let current_state = orchestrator.get_state();

    println!("✅ [Rust] Cycle session ended");

    Ok(current_state)
}

/// Get the current cycle state
#[tauri::command]
pub async fn get_cycle_state(state: State<'_, AppState>) -> Result<CycleState, String> {
    println!("📊 [Rust] get_cycle_state called");

    let cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_ref()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let current_state = orchestrator.get_state();

    Ok(current_state)
}

/// Handle timer tick (should be called every second by frontend)
#[tauri::command]
pub async fn cycle_tick(state: State<'_, AppState>, app: AppHandle) -> Result<CycleState, String> {
    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let events = orchestrator.tick()?;

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    let current_state = orchestrator.get_state();

    Ok(current_state)
}

/// Reset the cycle counter
#[tauri::command]
pub async fn reset_cycle_count(state: State<'_, AppState>) -> Result<CycleState, String> {
    println!("🔄 [Rust] reset_cycle_count called");

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    orchestrator.reset_cycle_count();

    let current_state = orchestrator.get_state();

    println!("✅ [Rust] Cycle count reset");

    Ok(current_state)
}
