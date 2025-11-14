use crate::api_models::UserSettings as ApiUserSettings;
use crate::database::models::UserSettings;
use crate::state::AppState;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CycleConfig {
    pub focus_duration: i32,      // minutes
    pub break_duration: i32,      // minutes
    pub long_break_duration: i32, // minutes
    pub cycles_per_long_break: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrictModeConfig {
    pub strict_mode: bool,
    pub emergency_key_combination: Option<String>,
}

/// Validate cycle configuration
fn validate_cycle_config(config: &CycleConfig) -> Result<(), String> {
    // Validate focus duration (1-120 minutes)
    if config.focus_duration < 1 || config.focus_duration > 120 {
        return Err("Focus duration must be between 1 and 120 minutes".to_string());
    }

    // Validate break duration (1-60 minutes)
    if config.break_duration < 1 || config.break_duration > 60 {
        return Err("Break duration must be between 1 and 60 minutes".to_string());
    }

    // Validate long break duration (1-120 minutes)
    if config.long_break_duration < 1 || config.long_break_duration > 120 {
        return Err("Long break duration must be between 1 and 120 minutes".to_string());
    }

    // Validate cycles per long break (1-10)
    if config.cycles_per_long_break < 1 || config.cycles_per_long_break > 10 {
        return Err("Cycles per long break must be between 1 and 10".to_string());
    }

    // Validate that long break is longer than regular break
    if config.long_break_duration <= config.break_duration {
        return Err("Long break duration must be longer than regular break duration".to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn save_cycle_config(
    config: CycleConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "💾 [Rust] save_cycle_config called with config: {:?}",
        config
    );

    // Validate the configuration
    validate_cycle_config(&config)?;

    let now = Utc::now();

    // Convert minutes to seconds for storage
    let focus_duration_seconds = config.focus_duration * 60;
    let break_duration_seconds = config.break_duration * 60;
    let long_break_duration_seconds = config.long_break_duration * 60;

    // Update user settings with cycle configuration
    let result = state.database.with_connection(|conn| {
        conn.execute(
            r#"
            UPDATE user_settings 
            SET focus_duration = ?1,
                short_break_duration = ?2,
                long_break_duration = ?3,
                cycles_per_long_break_v2 = ?4,
                updated_at = ?5
            WHERE id = 1
            "#,
            params![
                focus_duration_seconds,
                break_duration_seconds,
                long_break_duration_seconds,
                config.cycles_per_long_break,
                now
            ],
        )
        .map_err(|e| crate::database::DatabaseError::Sqlite(e))
    });

    match result {
        Ok(_) => {
            println!("✅ [Rust] Cycle configuration saved successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to save cycle configuration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_cycle_config(state: State<'_, AppState>) -> Result<CycleConfig, String> {
    println!("📖 [Rust] get_cycle_config called");

    let result = state.database.with_connection(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT focus_duration, short_break_duration, long_break_duration, 
                       cycles_per_long_break_v2
                FROM user_settings 
                WHERE id = 1
                "#,
            )
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        let config = stmt
            .query_row([], |row| {
                Ok(CycleConfig {
                    focus_duration: row.get::<_, i32>("focus_duration")? / 60, // Convert seconds to minutes
                    break_duration: row.get::<_, i32>("short_break_duration")? / 60,
                    long_break_duration: row.get::<_, i32>("long_break_duration")? / 60,
                    cycles_per_long_break: row.get("cycles_per_long_break_v2")?,
                })
            })
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        Ok(config)
    });

    match result {
        Ok(config) => {
            println!("✅ [Rust] Cycle configuration retrieved: {:?}", config);
            Ok(config)
        }
        Err(e) => {
            let error_msg = format!("Failed to get cycle configuration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_user_settings(state: State<'_, AppState>) -> Result<UserSettings, String> {
    println!("📖 [Rust] get_user_settings called");

    let result = state.database.with_connection(|conn| {
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
    });

    match result {
        Ok(settings) => {
            println!("✅ [Rust] User settings retrieved successfully");
            Ok(settings)
        }
        Err(e) => {
            let error_msg = format!("Failed to get user settings: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn update_user_name(user_name: String, state: State<'_, AppState>) -> Result<(), String> {
    println!("💾 [Rust] update_user_name called with name: {}", user_name);

    let now = Utc::now();

    let result = state.database.with_connection(|conn| {
        conn.execute(
            "UPDATE user_settings SET user_name = ?1, updated_at = ?2 WHERE id = 1",
            params![user_name, now],
        )
        .map_err(|e| crate::database::DatabaseError::Sqlite(e))
    });

    match result {
        Ok(_) => {
            println!("✅ [Rust] User name updated successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to update user name: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn save_strict_mode_config(
    config: StrictModeConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "💾 [Rust] save_strict_mode_config called with config: {:?}",
        config
    );

    let now = Utc::now();

    // Update user settings with strict mode configuration
    let result = state.database.with_connection(|conn| {
        conn.execute(
            r#"
            UPDATE user_settings 
            SET strict_mode = ?1,
                emergency_key_combination = ?2,
                updated_at = ?3
            WHERE id = 1
            "#,
            params![config.strict_mode, config.emergency_key_combination, now],
        )
        .map_err(|e| crate::database::DatabaseError::Sqlite(e))
    });

    match result {
        Ok(_) => {
            println!("✅ [Rust] Strict mode configuration saved successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to save strict mode configuration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_strict_mode_config(
    state: State<'_, AppState>,
) -> Result<StrictModeConfig, String> {
    println!("📖 [Rust] get_strict_mode_config called");

    let result = state.database.with_connection(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT strict_mode, emergency_key_combination
                FROM user_settings 
                WHERE id = 1
                "#,
            )
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        let config = stmt
            .query_row([], |row| {
                Ok(StrictModeConfig {
                    strict_mode: row.get("strict_mode")?,
                    emergency_key_combination: row.get("emergency_key_combination")?,
                })
            })
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        Ok(config)
    });

    match result {
        Ok(config) => {
            println!(
                "✅ [Rust] Strict mode configuration retrieved: {:?}",
                config
            );
            Ok(config)
        }
        Err(e) => {
            let error_msg = format!("Failed to get strict mode configuration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreAlertConfig {
    pub pre_alert_seconds: i32,
    pub enabled: bool,
}

/// Update pre-alert configuration
#[tauri::command]
pub async fn update_pre_alert_config(
    config: PreAlertConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "💾 [Rust] update_pre_alert_config called with config: {:?}",
        config
    );

    // Validate pre-alert seconds (30-300 seconds, i.e., 30 seconds to 5 minutes)
    if config.pre_alert_seconds < 30 || config.pre_alert_seconds > 300 {
        return Err("Pre-alert time must be between 30 and 300 seconds".to_string());
    }

    let now = Utc::now();

    // If disabled, set to 0, otherwise use the configured value
    let pre_alert_value = if config.enabled {
        config.pre_alert_seconds
    } else {
        0
    };

    let result = state.database.with_connection(|conn| {
        conn.execute(
            "UPDATE user_settings SET pre_alert_seconds = ?1, updated_at = ?2 WHERE id = 1",
            params![pre_alert_value, now],
        )
        .map_err(|e| crate::database::DatabaseError::Sqlite(e))
    });

    match result {
        Ok(_) => {
            println!("✅ [Rust] Pre-alert configuration updated successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to update pre-alert configuration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

/// Get pre-alert configuration
#[tauri::command]
pub async fn get_pre_alert_config(state: State<'_, AppState>) -> Result<PreAlertConfig, String> {
    println!("📖 [Rust] get_pre_alert_config called");

    let result = state.database.with_connection(|conn| {
        let mut stmt = conn
            .prepare("SELECT pre_alert_seconds FROM user_settings WHERE id = 1")
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        let pre_alert_seconds: i32 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        Ok(PreAlertConfig {
            pre_alert_seconds: if pre_alert_seconds > 0 {
                pre_alert_seconds
            } else {
                120 // Default to 2 minutes if disabled
            },
            enabled: pre_alert_seconds > 0,
        })
    });

    match result {
        Ok(config) => {
            println!("✅ [Rust] Pre-alert configuration retrieved: {:?}", config);
            Ok(config)
        }
        Err(e) => {
            let error_msg = format!("Failed to get pre-alert configuration: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

/// Get all user settings
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<ApiUserSettings, String> {
    println!("📖 [Rust] get_settings called");

    // Get user settings from database
    let db_settings = match state.database.get_user_settings() {
        Ok(Some(settings)) => settings,
        Ok(None) => {
            // Return default settings if none exist
            return Ok(ApiUserSettings::default());
        }
        Err(e) => {
            let error_msg = format!("Failed to get user settings: {}", e);
            println!("❌ [Rust] {}", error_msg);
            return Err(error_msg);
        }
    };

    // Convert to API model
    let api_settings = ApiUserSettings {
        focus_duration: (db_settings.focus_duration / 60) as u32,
        short_break_duration: (db_settings.short_break_duration / 60) as u32,
        long_break_duration: (db_settings.long_break_duration / 60) as u32,
        cycles_per_long_break: db_settings.cycles_per_long_break as u32,
        pre_alert_seconds: db_settings.pre_alert_seconds as u32,
        strict_mode: db_settings.strict_mode,
        pin_hash: db_settings.pin_hash,
        emergency_key_combination: db_settings.emergency_key_combination,
        break_transition_seconds: db_settings.break_transition_seconds as u32,
    };

    println!("✅ [Rust] Settings retrieved successfully");
    Ok(api_settings)
}

/// Update all user settings including blocked apps and websites
#[tauri::command]
pub async fn update_settings(
    settings: ApiUserSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!("💾 [Rust] update_settings called");

    let now = Utc::now();

    // Get existing settings to preserve user_name, emergency_key_combination, and created_at
    let existing_settings = state
        .database
        .get_user_settings()
        .map_err(|e| format!("Failed to get existing settings: {}", e))?;

    // Convert API settings to database model
    let db_settings = UserSettings {
        id: 1,
        focus_duration: (settings.focus_duration * 60) as i32, // Convert minutes to seconds
        short_break_duration: (settings.short_break_duration * 60) as i32,
        long_break_duration: (settings.long_break_duration * 60) as i32,
        cycles_per_long_break: settings.cycles_per_long_break as i32,
        cycles_per_long_break_v2: settings.cycles_per_long_break as i32,
        pre_alert_seconds: settings.pre_alert_seconds as i32,
        strict_mode: settings.strict_mode,
        pin_hash: settings.pin_hash,
        user_name: existing_settings.as_ref().and_then(|s| s.user_name.clone()),
        emergency_key_combination: settings.emergency_key_combination.or_else(|| {
            existing_settings
                .as_ref()
                .and_then(|s| s.emergency_key_combination.clone())
        }),
        break_transition_seconds: existing_settings
            .as_ref()
            .map(|s| s.break_transition_seconds)
            .unwrap_or(10),
        created_at: existing_settings
            .as_ref()
            .map(|s| s.created_at)
            .unwrap_or(now),
        updated_at: now,
    };

    // Save user settings
    state
        .database
        .save_user_settings(&db_settings)
        .map_err(|e| format!("Failed to save user settings: {}", e))?;

    println!("✅ [Rust] Settings updated successfully");
    Ok(())
}
