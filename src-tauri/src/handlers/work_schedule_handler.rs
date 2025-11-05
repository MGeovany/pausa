use crate::database::models::WorkSchedule;
use crate::state::AppState;
use chrono::{Local, NaiveTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkScheduleConfig {
    pub use_work_schedule: bool,
    pub work_start_time: Option<String>,
    pub work_end_time: Option<String>,
    pub timezone: Option<String>,
}

/// Validate work schedule configuration
fn validate_work_schedule(config: &WorkScheduleConfig) -> Result<(), String> {
    if !config.use_work_schedule {
        return Ok(()); // No validation needed if not using work schedule
    }

    let start_time = config
        .work_start_time
        .as_ref()
        .ok_or("Start time is required when using work schedule")?;
    let end_time = config
        .work_end_time
        .as_ref()
        .ok_or("End time is required when using work schedule")?;

    // Validate time format
    let start_parsed = NaiveTime::parse_from_str(start_time, "%H:%M")
        .map_err(|_| format!("Invalid start time format: {}. Expected HH:MM", start_time))?;
    let end_parsed = NaiveTime::parse_from_str(end_time, "%H:%M")
        .map_err(|_| format!("Invalid end time format: {}. Expected HH:MM", end_time))?;

    // Validate that end time is after start time
    if end_parsed <= start_parsed {
        return Err("End time must be after start time".to_string());
    }

    // Validate reasonable work hours (not more than 16 hours)
    let duration = end_parsed.signed_duration_since(start_parsed);
    if duration.num_hours() > 16 {
        return Err("Work day cannot be longer than 16 hours".to_string());
    }

    Ok(())
}

/// Get current system timezone
fn get_system_timezone() -> String {
    // Try to get system timezone, fallback to "local"
    match iana_time_zone::get_timezone() {
        Ok(tz) => tz,
        Err(_) => "local".to_string(),
    }
}

#[tauri::command]
pub async fn save_work_schedule(
    config: WorkScheduleConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "💾 [Rust] save_work_schedule called with config: {:?}",
        config
    );

    // Validate the configuration
    validate_work_schedule(&config)?;

    let now = Utc::now();
    let timezone = config.timezone.unwrap_or_else(|| get_system_timezone());

    // Update or insert work schedule
    let result = state.database.with_connection(|conn| {
        conn.execute(
            r#"
            INSERT OR REPLACE INTO work_schedule 
            (id, user_id, use_work_schedule, work_start_time, work_end_time, timezone, updated_at)
            VALUES (1, 1, ?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                config.use_work_schedule,
                config.work_start_time,
                config.work_end_time,
                timezone,
                now
            ],
        )
        .map_err(|e| crate::database::DatabaseError::Sqlite(e))
    });

    match result {
        Ok(_) => {
            println!("✅ [Rust] Work schedule saved successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to save work schedule: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_work_schedule(state: State<'_, AppState>) -> Result<WorkSchedule, String> {
    println!("📖 [Rust] get_work_schedule called");

    let result = state.database.with_connection(|conn| {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, user_id, use_work_schedule, work_start_time, work_end_time, 
                       timezone, created_at, updated_at
                FROM work_schedule 
                WHERE user_id = 1
                ORDER BY id DESC 
                LIMIT 1
                "#,
            )
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        let work_schedule = stmt
            .query_row([], |row| WorkSchedule::from_row(row))
            .map_err(|e| crate::database::DatabaseError::Sqlite(e))?;

        Ok(work_schedule)
    });

    match result {
        Ok(work_schedule) => {
            println!("✅ [Rust] Work schedule retrieved: {:?}", work_schedule);
            Ok(work_schedule)
        }
        Err(e) => {
            let error_msg = format!("Failed to get work schedule: {}", e);
            println!("❌ [Rust] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn is_within_work_hours(state: State<'_, AppState>) -> Result<bool, String> {
    println!("🕐 [Rust] is_within_work_hours called");

    let work_schedule = get_work_schedule(state).await?;

    if !work_schedule.use_work_schedule {
        println!("✅ [Rust] Work schedule not enabled, returning true");
        return Ok(true);
    }

    let start_time_str = work_schedule.work_start_time;
    let end_time_str = work_schedule.work_end_time;

    if start_time_str.is_none() || end_time_str.is_none() {
        println!("⚠️ [Rust] Work hours not configured, returning true");
        return Ok(true);
    }

    let start_time_str = start_time_str.unwrap();
    let end_time_str = end_time_str.unwrap();

    // Parse work hours
    let start_time = NaiveTime::parse_from_str(&start_time_str, "%H:%M")
        .map_err(|e| format!("Invalid start time format: {}", e))?;
    let end_time = NaiveTime::parse_from_str(&end_time_str, "%H:%M")
        .map_err(|e| format!("Invalid end time format: {}", e))?;

    // Get current time in local timezone
    let now = Local::now();
    let current_time = now.time();

    println!(
        "🕐 [Rust] Current time: {}, Work hours: {} - {}",
        current_time.format("%H:%M"),
        start_time.format("%H:%M"),
        end_time.format("%H:%M")
    );

    // Check if current time is within work hours
    let within_hours = if end_time > start_time {
        // Normal case: start < end (e.g., 09:00 - 17:00)
        current_time >= start_time && current_time <= end_time
    } else {
        // Overnight case: start > end (e.g., 22:00 - 06:00)
        current_time >= start_time || current_time <= end_time
    };

    println!("✅ [Rust] Within work hours: {}", within_hours);
    Ok(within_hours)
}

#[tauri::command]
pub async fn get_system_timezone_info() -> Result<String, String> {
    println!("🌍 [Rust] get_system_timezone_info called");

    let timezone = get_system_timezone();
    println!("✅ [Rust] System timezone: {}", timezone);

    Ok(timezone)
}

#[tauri::command]
pub async fn validate_work_hours(start_time: String, end_time: String) -> Result<bool, String> {
    println!(
        "✅ [Rust] validate_work_hours called: {} - {}",
        start_time, end_time
    );

    let config = WorkScheduleConfig {
        use_work_schedule: true,
        work_start_time: Some(start_time),
        work_end_time: Some(end_time),
        timezone: None,
    };

    match validate_work_schedule(&config) {
        Ok(_) => {
            println!("✅ [Rust] Work hours validation passed");
            Ok(true)
        }
        Err(e) => {
            println!("❌ [Rust] Work hours validation failed: {}", e);
            Err(e)
        }
    }
}
