use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{
    Session as DbSession, SessionStats as DbSessionStats, SessionType,
    UserSettings as DbUserSettings,
};

/// API model for user settings - simplified for frontend use
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub focus_duration: u32,       // minutes
    pub short_break_duration: u32, // minutes
    pub long_break_duration: u32,  // minutes
    pub cycles_per_long_break: u32,
    pub pre_alert_seconds: u32,
    pub strict_mode: bool,
    pub pin_hash: Option<String>,
    pub emergency_key_combination: Option<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            focus_duration: 25,      // 25 minutes
            short_break_duration: 5, // 5 minutes
            long_break_duration: 15, // 15 minutes
            cycles_per_long_break: 4,
            pre_alert_seconds: 120, // 2 minutes
            strict_mode: false,
            pin_hash: None,
            emergency_key_combination: None,
        }
    }
}

/// API model for active focus sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub duration: u32,  // total duration in seconds
    pub remaining: u32, // remaining time in seconds
    pub is_running: bool,
    pub is_strict: bool,
    pub state: SessionState,
}

/// Session state for the focus widget
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Running,
    PreAlert,
    Ending,
}

/// API model for break sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakSession {
    pub id: String,
    #[serde(rename = "type")]
    pub break_type: BreakType,
    pub duration: u32,  // total duration in seconds
    pub remaining: u32, // remaining time in seconds
    pub activity: BreakActivity,
    pub allow_emergency: bool,
}

/// Break type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakType {
    Short,
    Long,
}

/// Break activity suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakActivity {
    pub title: String,
    pub description: String,
    pub checklist: Vec<String>,
}

/// Session statistics for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    pub date: String,
    pub focus_minutes: u32,
    pub breaks_completed: u32,
    pub sessions_completed: u32,
    pub evasion_attempts: u32,
}

/// Conversion functions between database models and API models

impl From<DbUserSettings> for UserSettings {
    fn from(db_settings: DbUserSettings) -> Self {
        Self {
            focus_duration: (db_settings.focus_duration / 60) as u32, // Convert seconds to minutes
            short_break_duration: (db_settings.short_break_duration / 60) as u32,
            long_break_duration: (db_settings.long_break_duration / 60) as u32,
            cycles_per_long_break: db_settings.cycles_per_long_break as u32,
            pre_alert_seconds: db_settings.pre_alert_seconds as u32,
            strict_mode: db_settings.strict_mode,
            pin_hash: db_settings.pin_hash,
            emergency_key_combination: db_settings.emergency_key_combination,
        }
    }
}

impl From<UserSettings> for DbUserSettings {
    fn from(api_settings: UserSettings) -> Self {
        let now = Utc::now();
        Self {
            id: 1, // Default ID for single-user application
            focus_duration: (api_settings.focus_duration * 60) as i32, // Convert minutes to seconds
            short_break_duration: (api_settings.short_break_duration * 60) as i32,
            long_break_duration: (api_settings.long_break_duration * 60) as i32,
            cycles_per_long_break: api_settings.cycles_per_long_break as i32,
            cycles_per_long_break_v2: api_settings.cycles_per_long_break as i32, // Use same value
            pre_alert_seconds: api_settings.pre_alert_seconds as i32,
            strict_mode: api_settings.strict_mode,
            pin_hash: api_settings.pin_hash,
            user_name: None, // Not exposed in API model
            emergency_key_combination: None, // Not exposed in API model
            created_at: now,
            updated_at: now,
        }
    }
}

impl FocusSession {
    /// Create a new focus session from database session
    pub fn from_db_session(db_session: DbSession, current_time: DateTime<Utc>) -> Option<Self> {
        if db_session.session_type != SessionType::Focus {
            return None;
        }

        let elapsed_seconds = if let Some(end_time) = db_session.end_time {
            (end_time - db_session.start_time).num_seconds() as u32
        } else {
            (current_time - db_session.start_time).num_seconds() as u32
        };

        let duration = db_session.planned_duration as u32;
        let remaining = if elapsed_seconds >= duration {
            0
        } else {
            duration - elapsed_seconds
        };

        let is_running = db_session.end_time.is_none() && !db_session.completed;

        // Determine session state based on remaining time and completion
        let state = if !is_running {
            SessionState::Idle
        } else if remaining <= 120 && remaining > 0 {
            // Pre-alert in last 2 minutes
            SessionState::PreAlert
        } else if remaining == 0 {
            SessionState::Ending
        } else {
            SessionState::Running
        };

        Some(Self {
            id: db_session.id,
            start_time: db_session.start_time,
            duration,
            remaining,
            is_running,
            is_strict: db_session.strict_mode,
            state,
        })
    }

    /// Convert to database session model
    pub fn to_db_session(&self) -> DbSession {
        let end_time = if !self.is_running {
            Some(
                self.start_time
                    + chrono::Duration::seconds(self.duration as i64 - self.remaining as i64),
            )
        } else {
            None
        };

        DbSession {
            id: self.id.clone(),
            session_type: SessionType::Focus,
            start_time: self.start_time,
            end_time,
            planned_duration: self.duration as i32,
            actual_duration: if end_time.is_some() {
                Some((self.duration - self.remaining) as i32)
            } else {
                None
            },
            strict_mode: self.is_strict,
            completed: !self.is_running && self.remaining == 0,
            notes: None,
            created_at: self.start_time,
            within_work_hours: false, // Default value, should be set by orchestrator
            cycle_number: None, // Default value, should be set by orchestrator
            is_long_break: false, // Focus sessions are not breaks
        }
    }
}

impl BreakSession {
    /// Create a new break session
    pub fn new(break_type: BreakType, duration_minutes: u32, allow_emergency: bool) -> Self {
        let activity = match break_type {
            BreakType::Short => BreakActivity {
                title: "Quick Refresh".to_string(),
                description: "Take a moment to recharge with these quick activities".to_string(),
                checklist: vec![
                    "💧 Drink a glass of water".to_string(),
                    "👀 Look away from the screen (20-20-20 rule)".to_string(),
                    "🧘 Take 3 deep breaths".to_string(),
                    "🚶 Stand up and stretch".to_string(),
                ],
            },
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
        };

        Self {
            id: Uuid::new_v4().to_string(),
            break_type,
            duration: duration_minutes * 60, // Convert to seconds
            remaining: duration_minutes * 60,
            activity,
            allow_emergency,
        }
    }

    /// Convert to database session model
    pub fn to_db_session(&self, start_time: DateTime<Utc>) -> DbSession {
        let session_type = match self.break_type {
            BreakType::Short => SessionType::ShortBreak,
            BreakType::Long => SessionType::LongBreak,
        };

        DbSession {
            id: self.id.clone(),
            session_type,
            start_time,
            end_time: None,
            planned_duration: self.duration as i32,
            actual_duration: None,
            strict_mode: false, // Breaks don't use strict mode
            completed: false,
            notes: None,
            created_at: start_time,
            within_work_hours: false, // Default value, should be set by orchestrator
            cycle_number: None, // Default value, should be set by orchestrator
            is_long_break: matches!(self.break_type, BreakType::Long),
        }
    }
}

impl From<DbSessionStats> for SessionStats {
    fn from(db_stats: DbSessionStats) -> Self {
        Self {
            date: db_stats.date,
            focus_minutes: db_stats.focus_minutes,
            breaks_completed: db_stats.breaks_completed,
            sessions_completed: db_stats.sessions_completed,
            evasion_attempts: db_stats.evasion_attempts,
        }
    }
}

