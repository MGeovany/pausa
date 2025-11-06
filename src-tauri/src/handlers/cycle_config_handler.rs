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
