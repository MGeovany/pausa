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

/// Start a focus session with optional work hours override
#[tauri::command]
pub async fn start_focus_session(
    override_work_hours: Option<bool>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<CycleState, String> {
    let override_flag = override_work_hours.unwrap_or(false);
    println!(
        "▶️ [Rust] start_focus_session called (override: {})",
        override_flag
    );

    // Reload settings from database to ensure we have the latest configuration
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

    // Create updated cycle config
    let config = CycleConfig::from_user_settings(user_settings.clone(), work_schedule);

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    // Update orchestrator with latest configuration
    orchestrator.update_config(config);

    let events = orchestrator.start_focus_session_with_override(override_flag)?;

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

    // Reload settings from database to ensure we have the latest configuration
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

    // Create updated cycle config
    let config = CycleConfig::from_user_settings(user_settings.clone(), work_schedule);

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    // Update orchestrator with latest configuration
    orchestrator.update_config(config);

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

/// Get work schedule information for UI display
#[tauri::command]
pub async fn get_work_schedule_info(
    state: State<'_, AppState>,
) -> Result<Option<crate::cycle_orchestrator::WorkScheduleInfo>, String> {
    println!("📅 [Rust] get_work_schedule_info called");

    let cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_ref()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let info = orchestrator.get_work_schedule_info();

    println!("✅ [Rust] Work schedule info retrieved: {:?}", info);

    Ok(info)
}

/// Get work hours compliance statistics
#[tauri::command]
pub async fn get_work_hours_stats(
    days: Option<u32>,
    state: State<'_, AppState>,
) -> Result<crate::database::models::WorkHoursStats, String> {
    let days = days.unwrap_or(30); // Default to last 30 days
    println!(
        "📊 [Rust] get_work_hours_stats called for last {} days",
        days
    );

    let stats = state
        .database
        .with_connection(|conn| {
            // Calculate date range
            let now = chrono::Utc::now();
            let start_date = now - chrono::Duration::days(days as i64);

            // Query sessions within date range
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT 
                        COUNT(*) as total_sessions,
                        SUM(CASE WHEN within_work_hours = 1 THEN 1 ELSE 0 END) as within_hours,
                        SUM(CASE WHEN within_work_hours = 0 THEN 1 ELSE 0 END) as outside_hours,
                        SUM(CASE WHEN within_work_hours = 1 AND session_type = 'focus' AND completed = 1 
                            THEN actual_duration ELSE 0 END) as focus_minutes_within,
                        SUM(CASE WHEN within_work_hours = 0 AND session_type = 'focus' AND completed = 1 
                            THEN actual_duration ELSE 0 END) as focus_minutes_outside
                    FROM sessions
                    WHERE start_time >= ?1 AND session_type = 'focus'
                    "#,
                )
                .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

            let result = stmt.query_row([start_date], |row| {
                let total: u32 = row.get(0).unwrap_or(0);
                let within: u32 = row.get(1).unwrap_or(0);
                let outside: u32 = row.get(2).unwrap_or(0);
                let focus_within_seconds: i32 = row.get(3).unwrap_or(0);
                let focus_outside_seconds: i32 = row.get(4).unwrap_or(0);

                let compliance_percentage = if total > 0 {
                    (within as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                Ok(crate::database::models::WorkHoursStats {
                    total_sessions: total,
                    within_work_hours: within,
                    outside_work_hours: outside,
                    compliance_percentage,
                    total_focus_minutes_within: (focus_within_seconds / 60) as u32,
                    total_focus_minutes_outside: (focus_outside_seconds / 60) as u32,
                    period_start: start_date.format("%Y-%m-%d").to_string(),
                    period_end: now.format("%Y-%m-%d").to_string(),
                })
            });

            result.map_err(|e| crate::database::DatabaseError::Sqlite(e))
        })
        .map_err(|e| format!("Failed to get work hours stats: {}", e))?;

    println!("✅ [Rust] Work hours stats retrieved: {:?}", stats);

    Ok(stats)
}
