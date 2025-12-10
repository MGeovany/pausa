use crate::api_models::{BreakActivity, BreakSession, BreakType};
use crate::cycle_orchestrator::{CycleConfig, CycleOrchestrator, CyclePhase, CycleState};
use crate::database::models::{Session, SessionType, UserSettings, WorkSchedule};
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

#[cfg(target_os = "macos")]
use crate::menu_bar_text;

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeCycleRequest {
    pub force_reload: Option<bool>,
}

/// Helper function to format time in MM:SS format
fn format_time(seconds: u32) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

// Icon generation functions commented out until Tauri supports text in menu bar
// See: https://github.com/tauri-apps/tao/issues/65
// When support is added, we can uncomment and use these functions to generate
// dynamic icons with text rendered on them.

/// Update the tray icon with text showing the timer
/// On macOS, we use native NSStatusItem to show text directly in menu bar
/// On other platforms, we fall back to tooltip
fn update_tray_icon_with_text(app: &AppHandle, state: &CycleState) {
    let text = match state.phase {
        CyclePhase::Idle => "Pausa".to_string(),
        CyclePhase::Focus => {
            if state.is_running {
                format_time(state.remaining_seconds)
            } else {
                format!("{} ⏸", format_time(state.remaining_seconds))
            }
        }
        CyclePhase::ShortBreak => {
            if state.is_running {
                format!("☕ {}", format_time(state.remaining_seconds))
            } else {
                format!("☕ {} ⏸", format_time(state.remaining_seconds))
            }
        }
        CyclePhase::LongBreak => {
            if state.is_running {
                format!("☕ {}", format_time(state.remaining_seconds))
            } else {
                format!("☕ {} ⏸", format_time(state.remaining_seconds))
            }
        }
    };

    // Try to update native menu bar text on macOS
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = menu_bar_text::update_menu_bar_text(&text) {
            update_tray_tooltip_fallback(app, state);
        }
    }

    // Always update tooltip as well (for non-macOS and as fallback)
    #[cfg(not(target_os = "macos"))]
    {
        update_tray_tooltip_fallback(app, state);
    }
}

/// Fallback: Update the tray icon tooltip with the current cycle state
fn update_tray_tooltip_fallback(app: &AppHandle, state: &CycleState) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let tooltip = match state.phase {
            CyclePhase::Idle => "Pausa - Inactivo".to_string(),
            CyclePhase::Focus => {
                if state.is_running {
                    format!("Pausa - Enfoque: {}", format_time(state.remaining_seconds))
                } else {
                    format!("Pausa - Enfoque (pausado): {}", format_time(state.remaining_seconds))
                }
            }
            CyclePhase::ShortBreak => {
                if state.is_running {
                    format!("Pausa - Descanso corto: {}", format_time(state.remaining_seconds))
                } else {
                    format!("Pausa - Descanso corto (pausado): {}", format_time(state.remaining_seconds))
                }
            }
            CyclePhase::LongBreak => {
                if state.is_running {
                    format!("Pausa - Descanso largo: {}", format_time(state.remaining_seconds))
                } else {
                    format!("Pausa - Descanso largo (pausado): {}", format_time(state.remaining_seconds))
                }
            }
        };

        if let Err(e) = tray.set_tooltip(Some(&tooltip)) {
            eprintln!("⚠️ [CycleHandler] Failed to update tray tooltip: {}", e);
        }
    }
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
                       break_transition_seconds, created_at, updated_at
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
    notification_service.set_user_name(user_settings.user_name.clone());

    // Initialize StrictModeOrchestrator if strict mode is enabled
    println!(
        "🔒 [initialize_cycle_orchestrator] Checking strict mode setting: {}",
        user_settings.strict_mode
    );

    if user_settings.strict_mode {
        use crate::strict_mode::{StrictModeConfig, StrictModeOrchestrator};
        use crate::window_manager::WindowManager;
        use std::sync::{Arc, Mutex as StdMutex};

        println!("🔒 [initialize_cycle_orchestrator] Strict mode is ENABLED, initializing StrictModeOrchestrator");

        let strict_config = StrictModeConfig {
            enabled: user_settings.strict_mode,
            emergency_key_combination: user_settings.emergency_key_combination.clone(),
            transition_countdown_seconds: user_settings.break_transition_seconds as u32,
        };

        println!("🔒 [initialize_cycle_orchestrator] StrictModeConfig: enabled={}, emergency_key={:?}, transition_seconds={}", 
            strict_config.enabled, strict_config.emergency_key_combination, strict_config.transition_countdown_seconds);

        // Create window manager (will be properly initialized in future tasks)
        let window_manager = Arc::new(StdMutex::new(WindowManager::new(state.app_handle.clone())));

        let mut strict_orchestrator =
            StrictModeOrchestrator::new(strict_config, state.app_handle.clone(), window_manager);

        // Restore state from database (in case app was closed during strict mode)
        println!("📂 [initialize_cycle_orchestrator] Restoring strict mode state from database...");
        if let Err(e) = strict_orchestrator.restore_state_from_database() {
            eprintln!(
                "⚠️ [initialize_cycle_orchestrator] Failed to restore strict mode state: {}",
                e
            );
        } else {
            println!("✅ [initialize_cycle_orchestrator] Strict mode state restored successfully");
        }

        // IMPORTANT: Activate strict mode immediately after creation (if not already active from restored state)
        if !strict_orchestrator.is_active() {
            println!("🔒 [initialize_cycle_orchestrator] Activating strict mode...");
            if let Err(e) = strict_orchestrator.activate() {
                eprintln!(
                    "❌ [initialize_cycle_orchestrator] Failed to activate strict mode: {}",
                    e
                );
            } else {
                println!("✅ [initialize_cycle_orchestrator] Strict mode activated successfully");
            }
        } else {
            println!(
                "ℹ️ [initialize_cycle_orchestrator] Strict mode already active from restored state"
            );
        }

        let mut strict_mode_orchestrator = state.strict_mode_orchestrator.lock().await;
        *strict_mode_orchestrator = Some(strict_orchestrator);

        println!("✅ [initialize_cycle_orchestrator] StrictModeOrchestrator initialized and stored in state");
    } else {
        println!("ℹ️ [initialize_cycle_orchestrator] Strict mode is DISABLED, skipping StrictModeOrchestrator initialization");
    }

    println!("✅ [Rust] Cycle orchestrator initialized");

    // Update tray icon with text showing timer
    update_tray_icon_with_text(&app, &current_state);

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
                       break_transition_seconds, created_at, updated_at
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

    // Save values we need before moving config
    let focus_duration = config.focus_duration;
    let strict_mode = config.strict_mode;

    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    // Update orchestrator with latest configuration
    orchestrator.update_config(config);

    let events = orchestrator.start_focus_session_with_override(override_flag)?;

    let current_state = orchestrator.get_state();

    // Release the cycle orchestrator lock before handling strict mode
    drop(cycle_orchestrator);

    // Handle strict mode events if strict mode is active
    println!(
        "🔒 [start_focus_session] Checking strict mode - strict_mode flag: {}",
        strict_mode
    );

    let mut strict_mode_orchestrator = state.strict_mode_orchestrator.lock().await;

    if let Some(strict_orchestrator) = strict_mode_orchestrator.as_mut() {
        println!("🔒 [start_focus_session] StrictModeOrchestrator exists");
        println!(
            "🔒 [start_focus_session] Is strict mode active? {}",
            strict_orchestrator.is_active()
        );

        if strict_orchestrator.is_active() {
            println!(
                "🔒 [start_focus_session] Processing {} cycle events for strict mode",
                events.len()
            );
            for event in &events {
                println!("🔒 [start_focus_session] Handling event: {:?}", event);
                if let Err(e) = strict_orchestrator.handle_cycle_event(event) {
                    eprintln!(
                        "❌ [start_focus_session] Failed to handle strict mode event: {}",
                        e
                    );
                } else {
                    println!("✅ [start_focus_session] Successfully handled strict mode event");
                }
            }
        } else {
            println!("⚠️ [start_focus_session] Strict mode orchestrator exists but is NOT active");
            println!("💡 [start_focus_session] Hint: You may need to activate strict mode first");
        }
    } else {
        println!("⚠️ [start_focus_session] StrictModeOrchestrator is None (not initialized)");
        println!(
            "💡 [start_focus_session] Strict mode in settings: {}",
            strict_mode
        );
    }
    drop(strict_mode_orchestrator);

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    // Save session to database
    if let Some(ref session_id) = current_state.session_id {
        let session = Session {
            id: session_id.clone(),
            session_type: SessionType::Focus,
            start_time: current_state.started_at.unwrap_or_else(Utc::now),
            end_time: None,
            planned_duration: focus_duration as i32,
            actual_duration: None,
            strict_mode,
            completed: false,
            notes: None,
            created_at: Utc::now(),
            within_work_hours: current_state.within_work_hours,
            cycle_number: Some(current_state.cycle_count as i32),
            is_long_break: false,
        };

        if let Err(e) = state.database.create_session(&session) {
            eprintln!("Failed to save session to database: {}", e);
        }
    }

    // Send focus start notification
    let notification_service = state.notification_service.lock().await;
    notification_service.notify_focus_start(&app);

    // Update tray icon with text showing timer
    update_tray_icon_with_text(&app, &current_state);

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
                       break_transition_seconds, created_at, updated_at
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

    // Save values we need before moving config
    let break_duration = config.break_duration;
    let long_break_duration = config.long_break_duration;
    let strict_mode = config.strict_mode;

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

    // Save session to database
    if let Some(ref session_id) = current_state.session_id {
        let session_type = match current_state.phase {
            CyclePhase::LongBreak => SessionType::LongBreak,
            CyclePhase::ShortBreak => SessionType::ShortBreak,
            _ => SessionType::ShortBreak, // fallback
        };

        let is_long_break = current_state.phase == CyclePhase::LongBreak;
        let duration = if is_long_break {
            long_break_duration
        } else {
            break_duration
        };

        let session = Session {
            id: session_id.clone(),
            session_type,
            start_time: current_state.started_at.unwrap_or_else(Utc::now),
            end_time: None,
            planned_duration: duration as i32,
            actual_duration: None,
            strict_mode,
            completed: false,
            notes: None,
            created_at: Utc::now(),
            within_work_hours: current_state.within_work_hours,
            cycle_number: Some(current_state.cycle_count as i32),
            is_long_break,
        };

        if let Err(e) = state.database.create_session(&session) {
            eprintln!("Failed to save break session to database: {}", e);
        }
    }

    // Send appropriate break notification based on phase
    let notification_service = state.notification_service.lock().await;
    match current_state.phase {
        CyclePhase::LongBreak => notification_service.notify_long_break_start(&app),
        _ => notification_service.notify_break_start(&app),
    };

    // Update tray icon with text showing timer
    update_tray_icon_with_text(&app, &current_state);

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

    // Update tray icon with text showing timer
    let app_handle = state.app_handle.clone();
    update_tray_icon_with_text(&app_handle, &current_state);

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

    // Update tray icon with text showing timer
    let app_handle = state.app_handle.clone();
    update_tray_icon_with_text(&app_handle, &current_state);

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

    // Get the state before ending to save session info
    let state_before_end = orchestrator.get_state();
    let phase_before_end = state_before_end.phase.clone();
    let session_id_before_end = state_before_end.session_id.clone();
    let started_at_before_end = state_before_end.started_at;
    let planned_duration_before_end = state_before_end.remaining_seconds;

    let events = orchestrator.end_session(completed)?;

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    let current_state = orchestrator.get_state();

    // Update session in database if it was completed
    if completed && session_id_before_end.is_some() {
        let end_time = Utc::now();
        let actual_duration = if let Some(started_at) = started_at_before_end {
            Some((end_time - started_at).num_seconds() as i32)
        } else {
            Some(planned_duration_before_end as i32)
        };

        // Get existing session from database
        if let Ok(Some(mut db_session)) =
            state.database.get_session(&session_id_before_end.unwrap())
        {
            db_session.end_time = Some(end_time);
            db_session.actual_duration = actual_duration;
            db_session.completed = true;

            if let Err(e) = state.database.update_session(&db_session) {
                eprintln!("Failed to update session in database: {}", e);
            }
        }
    }

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

    // Update tray icon with text showing timer
    update_tray_icon_with_text(&app, &current_state);

    println!("✅ [Rust] Cycle session ended");

    Ok(current_state)
}

/// Get the current cycle state
#[tauri::command]
pub async fn get_cycle_state(state: State<'_, AppState>) -> Result<CycleState, String> {
    let cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_ref()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let current_state = orchestrator.get_state();

    Ok(current_state)
}

/// Get the current break session details (if a break is active)
#[tauri::command]
pub async fn get_current_break(
    state: State<'_, AppState>,
) -> Result<Option<BreakSession>, String> {
    let cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_ref()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    let cycle_state = orchestrator.get_state();

    match cycle_state.phase {
        CyclePhase::ShortBreak | CyclePhase::LongBreak => {
            let config = orchestrator.get_config();

            let (break_type, duration) = match cycle_state.phase {
                CyclePhase::LongBreak => (BreakType::Long, config.long_break_duration),
                _ => (BreakType::Short, config.break_duration),
            };

            // Mirror the frontend defaults so the overlay has meaningful content
            let activity = match break_type {
                BreakType::Long => BreakActivity {
                    title: "Extended Break".to_string(),
                    description: "Time for a longer break to fully recharge".to_string(),
                    checklist: vec![
                        "🚶‍♂️ Take a short walk".to_string(),
                        "💧 Hydrate with water or herbal tea".to_string(),
                        "🥗 Have a healthy snack".to_string(),
                        "🧘‍♀️ Do some light stretching or meditation".to_string(),
                        "🌱 Step outside for fresh air".to_string(),
                        "📱 Check in with a friend or family member".to_string(),
                    ],
                },
                BreakType::Short => BreakActivity {
                    title: "Quick Refresh".to_string(),
                    description: "Take a moment to recharge with these quick activities"
                        .to_string(),
                    checklist: vec![
                        "💧 Drink a glass of water".to_string(),
                        "👀 Look away from the screen (20-20-20 rule)".to_string(),
                        "🧘 Take 3 deep breaths".to_string(),
                        "🚶 Stand up and stretch".to_string(),
                    ],
                },
            };

            let id = cycle_state
                .session_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string());

            Ok(Some(BreakSession {
                id,
                break_type,
                duration,
                remaining: cycle_state.remaining_seconds,
                activity,
                // In strict mode we don't allow emergency overrides from the overlay UI
                allow_emergency: false,
            }))
        }
        _ => Ok(None),
    }
}

/// Handle timer tick (should be called every second by frontend)
#[tauri::command]
pub async fn cycle_tick(state: State<'_, AppState>, app: AppHandle) -> Result<CycleState, String> {
    let mut cycle_orchestrator = state.cycle_orchestrator.lock().await;

    let orchestrator = cycle_orchestrator
        .as_mut()
        .ok_or_else(|| "Cycle orchestrator not initialized".to_string())?;

    // Get state before tick to track session completion
    let state_before_tick = orchestrator.get_state();
    let session_id_before = state_before_tick.session_id.clone();
    let started_at_before = state_before_tick.started_at;
    let planned_duration_before = state_before_tick.remaining_seconds;

    let events = orchestrator.tick()?;

    // Get current state after tick
    let current_state = orchestrator.get_state();

    // Handle PhaseEnded events to update sessions in database
    for event in &events {
        match event {
            crate::cycle_orchestrator::CycleEvent::PhaseEnded { completed, phase } => {
                println!("📥 [CycleHandler] PhaseEnded event received: phase={:?}, completed={}, session_id_before={:?}", 
                    phase, completed, session_id_before);

                if *completed && session_id_before.is_some() {
                    let end_time = Utc::now();
                    let actual_duration = if let Some(started_at) = started_at_before {
                        Some((end_time - started_at).num_seconds() as i32)
                    } else {
                        Some(planned_duration_before as i32)
                    };

                    println!("💾 [CycleHandler] Updating session {}: end_time={:?}, actual_duration={:?}", 
                        session_id_before.as_ref().unwrap(), end_time, actual_duration);

                    // Get existing session from database and update it
                    match state
                        .database
                        .get_session(&session_id_before.as_ref().unwrap())
                    {
                        Ok(Some(mut db_session)) => {
                            println!(
                                "📝 [CycleHandler] Found session in DB: type={:?}, completed={}",
                                db_session.session_type, db_session.completed
                            );

                            db_session.end_time = Some(end_time);
                            db_session.actual_duration = actual_duration;
                            db_session.completed = true;

                            if let Err(e) = state.database.update_session(&db_session) {
                                eprintln!("❌ [CycleHandler] Failed to update completed session in database: {}", e);
                            } else {
                                println!(
                                    "✅ [CycleHandler] Successfully updated session {} in database",
                                    session_id_before.as_ref().unwrap()
                                );
                            }
                        }
                        Ok(None) => {
                            eprintln!(
                                "⚠️ [CycleHandler] Session {} not found in database",
                                session_id_before.as_ref().unwrap()
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "❌ [CycleHandler] Error getting session from database: {}",
                                e
                            );
                        }
                    }
                } else {
                    println!("⏭️ [CycleHandler] Skipping session update: completed={}, has_session_id={}", 
                        completed, session_id_before.is_some());
                }
            }
            crate::cycle_orchestrator::CycleEvent::PhaseStarted {
                phase,
                duration,
                cycle_count,
            } => {
                println!("📥 [CycleHandler] PhaseStarted event received: phase={:?}, duration={}, cycle_count={}", 
                    phase, duration, cycle_count);

                // Save new session when break starts automatically after focus
                if let Some(ref session_id) = current_state.session_id {
                    println!(
                        "🆔 [CycleHandler] Current session_id: {}, Previous session_id: {:?}",
                        session_id, session_id_before
                    );

                    // Check if this is a new session (not the one we had before)
                    if session_id_before.as_ref() != Some(session_id) {
                        println!("✨ [CycleHandler] New session detected, creating in database...");

                        let session_type = match phase {
                            CyclePhase::LongBreak => SessionType::LongBreak,
                            CyclePhase::ShortBreak => SessionType::ShortBreak,
                            CyclePhase::Focus => SessionType::Focus,
                            _ => SessionType::ShortBreak,
                        };

                        let is_long_break = *phase == CyclePhase::LongBreak;

                        println!("💾 [CycleHandler] Creating session: id={}, type={:?}, is_long_break={}, duration={}", 
                            session_id, session_type, is_long_break, duration);

                        let session = Session {
                            id: session_id.clone(),
                            session_type,
                            start_time: current_state.started_at.unwrap_or_else(Utc::now),
                            end_time: None,
                            planned_duration: *duration as i32,
                            actual_duration: None,
                            strict_mode: false, // We don't have access to config here, but it's ok
                            completed: false,
                            notes: None,
                            created_at: Utc::now(),
                            within_work_hours: current_state.within_work_hours,
                            cycle_number: Some(*cycle_count as i32),
                            is_long_break,
                        };

                        match state.database.create_session(&session) {
                            Ok(_) => {
                                println!(
                                    "✅ [CycleHandler] Successfully created session {} in database",
                                    session_id
                                );
                            }
                            Err(e) => {
                                eprintln!("❌ [CycleHandler] Failed to save auto-started session to database: {}", e);
                            }
                        }
                    } else {
                        println!(
                            "⏭️ [CycleHandler] Session {} already exists, skipping creation",
                            session_id
                        );
                    }
                } else {
                    println!("⚠️ [CycleHandler] No session_id in current state");
                }
            }
            _ => {}
        }
    }

    // Check for pre-alert events and send notifications
    let notification_service = state.notification_service.lock().await;

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

    // Handle strict mode events if strict mode is active
    let mut strict_mode_orchestrator = state.strict_mode_orchestrator.lock().await;
    if let Some(orchestrator) = strict_mode_orchestrator.as_mut() {
        if orchestrator.is_active() {
            for event in &events {
                if let Err(e) = orchestrator.handle_cycle_event(event) {
                    eprintln!("Failed to handle strict mode event: {}", e);
                }
            }
        }
    }
    drop(strict_mode_orchestrator); // Release lock before emitting events

    // Emit events to frontend
    for event in events {
        if let Err(e) = app.emit("cycle-event", &event) {
            eprintln!("Failed to emit cycle event: {}", e);
        }
    }

    // Update tray icon with text showing timer
    update_tray_icon_with_text(&app, &current_state);

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
