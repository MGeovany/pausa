use crate::onboarding::{
    create_post_onboarding_backup, create_pre_onboarding_backup, validate_step_data,
    OnboardingManager, OnboardingStep, OnboardingValidator,
};
use std::sync::Mutex;
use tauri::{Manager, State};

#[tauri::command]
pub async fn start_onboarding(
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<OnboardingStep, String> {
    println!("🚀 [Rust] start_onboarding called");

    let manager: std::sync::MutexGuard<'_, OnboardingManager> = state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    let current_step = manager.get_current_step().clone();
    println!("✅ [Rust] Current onboarding step: {:?}", current_step);

    Ok(current_step)
}

#[tauri::command]
pub async fn next_onboarding_step(
    step_data: Option<serde_json::Value>,
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<OnboardingStep, String> {
    println!(
        "🔄 [Rust] next_onboarding_step called with data: {:?}",
        step_data
    );

    let mut manager = state.lock().map_err(|e| {
        let error_msg = format!("Failed to acquire onboarding manager lock: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    let current_step = manager.get_current_step().clone();
    println!(
        "📍 [Rust] Current step before navigation: {:?}",
        current_step
    );

    // Validate and store step data if provided
    if let Some(data) = step_data {
        if data.is_null() {
            println!("⚠️ [Rust] Received null step data, skipping storage");
        } else {
            println!(
                "💾 [Rust] Validating and storing step data for {:?}: {:?}",
                current_step, data
            );

            // Validate step data before storing
            let step_name = format!("{:?}", current_step);
            if let Err(validation_errors) = validate_step_data(&step_name, &data) {
                let error_messages: Vec<String> =
                    validation_errors.iter().map(|e| e.to_string()).collect();
                let error_msg = format!("Validation failed: {}", error_messages.join(", "));
                println!("❌ [Rust] {}", error_msg);
                return Err(error_msg);
            }

            manager.set_step_data(current_step, data).map_err(|e| {
                let error_msg = format!("Failed to store step data: {}", e);
                println!("❌ [Rust] {}", error_msg);
                error_msg
            })?;
        }
    }

    // Attempt navigation
    let next_step = manager.next_step().map_err(|e| {
        let error_msg = format!("Navigation failed: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    println!(
        "✅ [Rust] Successfully advanced to next step: {:?}",
        next_step
    );
    Ok(next_step)
}

#[tauri::command]
pub async fn previous_onboarding_step(
    state: State<'_, Mutex<OnboardingManager>>,
) -> Result<OnboardingStep, String> {
    println!("🔙 [Rust] previous_onboarding_step called");

    let mut manager = state.lock().map_err(|e| {
        let error_msg = format!("Failed to acquire onboarding manager lock: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    let current_step = manager.get_current_step().clone();
    println!(
        "📍 [Rust] Current step before backward navigation: {:?}",
        current_step
    );

    // Attempt backward navigation
    let previous_step = manager.previous_step().map_err(|e| {
        let error_msg = format!("Backward navigation failed: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    println!(
        "✅ [Rust] Successfully moved to previous step: {:?}",
        previous_step
    );
    Ok(previous_step)
}

#[tauri::command]
pub async fn complete_onboarding(
    final_config: serde_json::Value,
    onboarding_state: State<'_, Mutex<OnboardingManager>>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<(), String> {
    println!(
        "🎉 [Rust] complete_onboarding called with config: {:?}",
        final_config
    );

    // Comprehensive validation of final configuration
    let mut validator = OnboardingValidator::new();
    if let Err(validation_errors) = validator.validate_configuration(&final_config) {
        let error_messages: Vec<String> = validation_errors.iter().map(|e| e.to_string()).collect();
        let error_msg = format!(
            "Configuration validation failed: {}",
            error_messages.join("; ")
        );
        println!("❌ [Rust] {}", error_msg);
        return Err(error_msg);
    }

    // Create backup before completing onboarding
    if let Ok(app_data_dir) = app_state.app_handle.path().app_data_dir() {
        match create_pre_onboarding_backup(&app_state.database, &app_data_dir) {
            Ok(backup_id) => {
                println!("✅ [Rust] Pre-onboarding backup created: {}", backup_id);
            }
            Err(e) => {
                println!("⚠️ [Rust] Failed to create pre-onboarding backup: {}", e);
                // Continue anyway - backup failure shouldn't block onboarding
            }
        }
    }

    let mut manager = onboarding_state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    // Store the final configuration
    manager.set_step_data(OnboardingStep::Complete, final_config.clone())?;

    // Mark as complete
    if !manager.is_complete() {
        manager.next_step()?; // This should set is_complete to true
    }

    // Save onboarding completion to database
    let config_json = serde_json::to_string(&final_config).map_err(|e| {
        let error_msg = format!("Failed to serialize config: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    app_state
        .database
        .save_onboarding_completion("1.0", Some(&config_json))
        .map_err(|e| {
            let error_msg = format!("Failed to save onboarding completion: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?;

    // Create backup after completing onboarding
    if let Ok(app_data_dir) = app_state.app_handle.path().app_data_dir() {
        match create_post_onboarding_backup(&app_state.database, &app_data_dir) {
            Ok(backup_id) => {
                println!("✅ [Rust] Post-onboarding backup created: {}", backup_id);
            }
            Err(e) => {
                println!("⚠️ [Rust] Failed to create post-onboarding backup: {}", e);
                // Continue anyway - backup failure shouldn't block onboarding
            }
        }
    }

    println!("✅ [Rust] Onboarding completed and saved to database successfully");

    Ok(())
}

#[tauri::command]
pub async fn get_onboarding_status(
    onboarding_state: State<'_, Mutex<OnboardingManager>>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<bool, String> {
    println!("📊 [Rust] get_onboarding_status called");

    // First check the database for persistent onboarding completion
    let is_db_complete = app_state.database.is_onboarding_completed().map_err(|e| {
        let error_msg = format!("Failed to check onboarding completion in database: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    if is_db_complete {
        println!("✅ [Rust] Onboarding completed (from database)");
        return Ok(true);
    }

    // If not in database, check the in-memory manager
    let manager = onboarding_state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    let is_complete = manager.is_complete();
    println!(
        "✅ [Rust] Onboarding status: {}",
        if is_complete {
            "complete"
        } else {
            "incomplete"
        }
    );

    Ok(is_complete)
}
#[tauri::command]
pub async fn is_first_launch(app_state: State<'_, crate::state::AppState>) -> Result<bool, String> {
    println!("🔍 [Rust] is_first_launch called");

    // The most reliable way to check if it's first launch is to see if onboarding has been completed
    // If onboarding hasn't been completed, it means the user needs to go through onboarding
    let onboarding_completed = app_state.database.is_onboarding_completed().map_err(|e| {
        let error_msg = format!("Failed to check onboarding completion: {}", e);
        println!("❌ [Rust] {}", error_msg);
        error_msg
    })?;

    // First launch = onboarding not completed
    let is_first = !onboarding_completed;

    println!(
        "✅ [Rust] First launch check: onboarding_completed={}, is_first={}",
        onboarding_completed, is_first
    );

    Ok(is_first)
}

#[tauri::command]
pub async fn reset_onboarding_for_testing(
    onboarding_state: State<'_, Mutex<OnboardingManager>>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<(), String> {
    println!("🔄 [Rust] reset_onboarding_for_testing called");

    // Reset the in-memory onboarding manager
    let mut manager = onboarding_state.lock().map_err(|e| {
        println!("❌ [Rust] Error acquiring onboarding manager lock: {}", e);
        format!("Failed to acquire onboarding manager lock: {}", e)
    })?;

    *manager = crate::onboarding::OnboardingManager::new();
    println!("✅ [Rust] In-memory onboarding manager reset");

    // Clear onboarding completion from database for testing
    app_state
        .database
        .with_connection(|conn| {
            conn.execute("DELETE FROM onboarding_completion", [])
                .map_err(crate::database::DatabaseError::Sqlite)?;
            Ok(())
        })
        .map_err(|e| {
            let error_msg = format!("Failed to clear onboarding completion: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?;

    println!("✅ [Rust] Onboarding state reset for testing");
    Ok(())
}
#[tauri::command]
pub async fn apply_onboarding_config_to_settings(
    config: serde_json::Value,
    app_state: State<'_, crate::state::AppState>,
) -> Result<(), String> {
    println!(
        "⚙️ [Rust] apply_onboarding_config_to_settings called with config: {:?}",
        config
    );

    // Parse the onboarding configuration
    let focus_duration = config
        .get("focusDuration")
        .and_then(|v| v.as_u64())
        .unwrap_or(25) as i32
        * 60; // Convert minutes to seconds

    let break_duration = config
        .get("breakDuration")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as i32
        * 60; // Convert minutes to seconds

    let long_break_duration = config
        .get("longBreakDuration")
        .and_then(|v| v.as_u64())
        .unwrap_or(15) as i32
        * 60; // Convert minutes to seconds

    let cycles_per_long_break = config
        .get("cyclesPerLongBreak")
        .and_then(|v| v.as_u64())
        .unwrap_or(4) as i32;

    let strict_mode = config
        .get("strictMode")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let user_name = config
        .get("userName")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let emergency_key = config
        .get("emergencyKey")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Get existing user settings or create default
    let mut user_settings = app_state
        .database
        .get_user_settings()
        .map_err(|e| {
            let error_msg = format!("Failed to get user settings: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?
        .unwrap_or_default();

    // Update settings with onboarding configuration
    user_settings.focus_duration = focus_duration;
    user_settings.short_break_duration = break_duration;
    user_settings.long_break_duration = long_break_duration;
    user_settings.cycles_per_long_break_v2 = cycles_per_long_break;
    user_settings.strict_mode = strict_mode;
    user_settings.user_name = user_name;
    user_settings.emergency_key_combination = emergency_key;
    user_settings.updated_at = chrono::Utc::now();

    // Save updated settings
    app_state
        .database
        .save_user_settings(&user_settings)
        .map_err(|e| {
            let error_msg = format!("Failed to save user settings: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?;

    println!("✅ [Rust] Onboarding configuration applied to user settings successfully");

    // Also save work schedule if provided
    if let Some(work_schedule_config) = config.get("workSchedule") {
        let use_work_schedule = work_schedule_config
            .get("useWorkSchedule")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let work_start_time = work_schedule_config
            .get("workStartTime")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let work_end_time = work_schedule_config
            .get("workEndTime")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create proper WorkScheduleConfig struct
        let work_schedule_config = crate::handlers::work_schedule_handler::WorkScheduleConfig {
            use_work_schedule,
            work_start_time,
            work_end_time,
            timezone: Some("local".to_string()),
        };

        // Call the work schedule handler directly
        crate::handlers::work_schedule_handler::save_work_schedule(
            work_schedule_config,
            app_state.clone(),
        )
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to save work schedule: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?;

        println!("✅ [Rust] Work schedule configuration applied successfully");
    }

    Ok(())
}

#[tauri::command]
pub async fn validate_onboarding_config(config: serde_json::Value) -> Result<(), String> {
    println!("🔍 [Rust] validate_onboarding_config called");

    let mut validator = OnboardingValidator::new();
    match validator.validate_configuration(&config) {
        Ok(()) => {
            println!("✅ [Rust] Configuration validation passed");
            Ok(())
        }
        Err(validation_errors) => {
            let error_messages: Vec<String> =
                validation_errors.iter().map(|e| e.to_string()).collect();
            let error_msg = format!("Validation errors: {}", error_messages.join("; "));
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn validate_step_config(
    step: String,
    step_data: serde_json::Value,
) -> Result<(), String> {
    println!("🔍 [Rust] validate_step_config called for step: {}", step);

    match validate_step_data(&step, &step_data) {
        Ok(()) => {
            println!("✅ [Rust] Step validation passed for: {}", step);
            Ok(())
        }
        Err(validation_errors) => {
            let error_messages: Vec<String> =
                validation_errors.iter().map(|e| e.to_string()).collect();
            let error_msg = format!("Step validation errors: {}", error_messages.join("; "));
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn create_configuration_backup(
    backup_type: String,
    description: Option<String>,
    app_state: State<'_, crate::state::AppState>,
) -> Result<String, String> {
    println!(
        "💾 [Rust] create_configuration_backup called with type: {}",
        backup_type
    );

    let backup_type_enum = match backup_type.as_str() {
        "manual" => crate::onboarding::BackupType::Manual,
        "pre_update" => crate::onboarding::BackupType::PreUpdate,
        _ => {
            return Err("Invalid backup type. Use 'manual' or 'pre_update'".to_string());
        }
    };

    let app_data_dir = app_state
        .app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let backup_manager = crate::onboarding::BackupManager::new(&app_data_dir)
        .map_err(|e| format!("Failed to create backup manager: {}", e))?;

    match backup_manager.create_backup(backup_type_enum, description, &app_state.database) {
        Ok(backup_id) => {
            println!("✅ [Rust] Configuration backup created: {}", backup_id);
            Ok(backup_id)
        }
        Err(e) => {
            let error_msg = format!("Failed to create backup: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn list_configuration_backups(
    app_state: State<'_, crate::state::AppState>,
) -> Result<Vec<(String, serde_json::Value)>, String> {
    println!("📋 [Rust] list_configuration_backups called");

    let app_data_dir = app_state
        .app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let backup_manager = crate::onboarding::BackupManager::new(&app_data_dir)
        .map_err(|e| format!("Failed to create backup manager: {}", e))?;

    match backup_manager.list_backups() {
        Ok(backups) => {
            let backup_list: Result<Vec<_>, _> = backups
                .into_iter()
                .map(|(id, metadata)| {
                    serde_json::to_value(metadata)
                        .map(|json| (id, json))
                        .map_err(|e| format!("Failed to serialize backup metadata: {}", e))
                })
                .collect();

            match backup_list {
                Ok(list) => {
                    println!("✅ [Rust] Listed {} backups", list.len());
                    Ok(list)
                }
                Err(e) => {
                    println!("❌ [Rust] {}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to list backups: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn restore_configuration_backup(
    backup_id: String,
    app_state: State<'_, crate::state::AppState>,
) -> Result<(), String> {
    println!(
        "🔄 [Rust] restore_configuration_backup called for: {}",
        backup_id
    );

    let app_data_dir = app_state
        .app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let backup_manager = crate::onboarding::BackupManager::new(&app_data_dir)
        .map_err(|e| format!("Failed to create backup manager: {}", e))?;

    match backup_manager.restore_backup(&backup_id, &app_state.database) {
        Ok(()) => {
            println!(
                "✅ [Rust] Configuration restored from backup: {}",
                backup_id
            );
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to restore backup: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_configuration_health_check(
    app_state: State<'_, crate::state::AppState>,
) -> Result<serde_json::Value, String> {
    println!("🏥 [Rust] get_configuration_health_check called");

    let mut health_report = serde_json::json!({
        "status": "healthy",
        "checks": {},
        "warnings": [],
        "errors": []
    });

    // Check onboarding completion
    match app_state.database.is_onboarding_completed() {
        Ok(completed) => {
            health_report["checks"]["onboarding_completed"] = serde_json::Value::Bool(completed);
            if !completed {
                health_report["warnings"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::Value::String(
                        "Onboarding not completed".to_string(),
                    ));
            }
        }
        Err(e) => {
            health_report["errors"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::Value::String(format!(
                    "Failed to check onboarding status: {}",
                    e
                )));
            health_report["status"] = serde_json::Value::String("error".to_string());
        }
    }

    // Check user settings
    match app_state.database.get_user_settings() {
        Ok(Some(_)) => {
            health_report["checks"]["user_settings_exist"] = serde_json::Value::Bool(true);
        }
        Ok(None) => {
            health_report["checks"]["user_settings_exist"] = serde_json::Value::Bool(false);
            health_report["warnings"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::Value::String(
                    "No user settings found".to_string(),
                ));
        }
        Err(e) => {
            health_report["errors"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::Value::String(format!(
                    "Failed to check user settings: {}",
                    e
                )));
            health_report["status"] = serde_json::Value::String("error".to_string());
        }
    }

    // Check database integrity
    match app_state.database.get_stats() {
        Ok(_) => {
            health_report["checks"]["database_accessible"] = serde_json::Value::Bool(true);
        }
        Err(e) => {
            health_report["errors"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::Value::String(format!(
                    "Database access error: {}",
                    e
                )));
            health_report["status"] = serde_json::Value::String("error".to_string());
        }
    }

    // Set overall status based on errors
    if !health_report["errors"].as_array().unwrap().is_empty() {
        health_report["status"] = serde_json::Value::String("error".to_string());
    } else if !health_report["warnings"].as_array().unwrap().is_empty() {
        health_report["status"] = serde_json::Value::String("warning".to_string());
    }

    println!("✅ [Rust] Configuration health check completed");
    Ok(health_report)
}
#[tauri::command]
pub async fn force_database_migration(
    app_state: State<'_, crate::state::AppState>,
) -> Result<(), String> {
    println!("🔧 [Rust] force_database_migration called");

    app_state
        .database
        .with_connection(|conn| {
            // Force run migrations
            crate::database::migrations::MigrationManager::migrate_to_current(conn)
        })
        .map_err(|e| {
            let error_msg = format!("Failed to run migration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            error_msg
        })?;

    println!("✅ [Rust] Database migration completed successfully");
    Ok(())
}
