use crate::cycle_orchestrator::{CycleConfig, CycleOrchestrator, CyclePhase, CycleState};
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
    _app: AppHandle,
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
    let config = CycleConfig::from_user_settings(user_settings.clone(), work_schedule);

    // Create orchestrator
    let orchestrator = CycleOrchestrator::new(config);

    let current_state = orchestrator.get_state();

    // Store in app state
    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;
    *cycle_orchestrator = Some(orchestrator);

    // Initialize notification service with user name
    let mut notification_service = state.notification_service.lock().await;
    notification_service.set_user_name(user_settings.user_name);

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

    // Send focus start notification
    let notification_service = state.notification_service.lock().await;
    notification_service.notify_focus_start(&app);

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

    let current_state = orchestrator.get_state();

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    // Send appropriate break notification based on phase
    let notification_service = state.notification_service.lock().await;
    match current_state.phase {
        CyclePhase::LongBreak => notification_service.notify_long_break_start(&app),
        _ => notification_service.notify_break_start(&app),
    };

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

    // Get the phase before ending to send appropriate notification
    let phase_before_end = orchestrator.get_state().phase.clone();

    let events = orchestrator.end_session(completed)?;

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    let current_state = orchestrator.get_state();

    // Send appropriate end notification if session was completed
    if completed {
        let notification_service = state.notification_service.lock().await;
        match phase_before_end {
            CyclePhase::Focus => notification_service.notify_focus_end(&app),
            CyclePhase::ShortBreak | CyclePhase::LongBreak => {
                notification_service.notify_break_end(&app)
            }
            _ => {}
        };
    }

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

    // Check for pre-alert events and send notifications
    let notification_service = state.notification_service.lock().await;
    let current_state = orchestrator.get_state();

    for event in &events {
        match event {
            crate::cycle_orchestrator::CycleEvent::PreAlert { remaining } => {
                // Send pre-alert notification for focus sessions
                let minutes_left = (remaining + 59) / 60; // Round up to nearest minute
                notification_service.notify_focus_warning(&app, minutes_left);
            }
            crate::cycle_orchestrator::CycleEvent::CycleCompleted { cycle_count } => {
                // Send cycle completed notification
                notification_service.notify_cycle_complete(&app, *cycle_count);
            }
            _ => {}
        }
    }

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

#[derive(Debug, Serialize, Deserialize)]
pub struct BypassAttemptLog {
    pub session_id: String,
    pub method: String,
    pub timestamp: String,
}

/// Log a bypass attempt during strict mode
#[tauri::command]
pub async fn log_bypass_attempt(
    session_id: String,
    method: String,
    timestamp: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "⚠️ [Rust] Bypass attempt logged - Session: {}, Method: {}, Time: {}",
        session_id, method, timestamp
    );

    // Store in database for persistent logging
    state
        .database
        .with_connection(|conn| {
            conn.execute(
                r#"
                INSERT INTO bypass_attempts (session_id, method, timestamp, created_at)
                VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)
                "#,
                rusqlite::params![session_id, method, timestamp],
            )
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

            Ok(())
        })
        .map_err(|e| format!("Failed to log bypass attempt: {}", e))?;

    println!("✅ [Rust] Bypass attempt logged to database");

    Ok(())
}
