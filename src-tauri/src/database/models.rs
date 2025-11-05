use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

/// User settings model matching the database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: i32,
    pub focus_duration: i32,
    pub short_break_duration: i32,
    pub long_break_duration: i32,
    pub cycles_per_long_break: i32,
    pub pre_alert_seconds: i32,
    pub strict_mode: bool,
    pub pin_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserSettings {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: 1,
            focus_duration: 1500,      // 25 minutes
            short_break_duration: 300, // 5 minutes
            long_break_duration: 900,  // 15 minutes
            cycles_per_long_break: 4,
            pre_alert_seconds: 120, // 2 minutes
            strict_mode: false,
            pin_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Block list item model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListItem {
    pub id: Option<i32>,
    pub item_type: BlockType,
    pub value: String,
    pub platform: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    App,
    Website,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::App => write!(f, "app"),
            BlockType::Website => write!(f, "website"),
        }
    }
}

impl std::str::FromStr for BlockType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "app" => Ok(BlockType::App),
            "website" => Ok(BlockType::Website),
            _ => Err(format!("Invalid block type: {}", s)),
        }
    }
}

/// Session model for focus and break sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub session_type: SessionType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub planned_duration: i32,
    pub actual_duration: Option<i32>,
    pub strict_mode: bool,
    pub completed: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Focus,
    ShortBreak,
    LongBreak,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Focus => write!(f, "focus"),
            SessionType::ShortBreak => write!(f, "short_break"),
            SessionType::LongBreak => write!(f, "long_break"),
        }
    }
}

impl std::str::FromStr for SessionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "focus" => Ok(SessionType::Focus),
            "short_break" => Ok(SessionType::ShortBreak),
            "long_break" => Ok(SessionType::LongBreak),
            _ => Err(format!("Invalid session type: {}", s)),
        }
    }
}

/// Evasion attempt model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvasionAttempt {
    pub id: Option<i32>,
    pub session_id: String,
    pub attempt_type: BlockType,
    pub blocked_item: String,
    pub timestamp: DateTime<Utc>,
}

/// Insights model for computed statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: Option<i32>,
    pub metric_key: String,
    pub metric_value: f64,
    pub period_start: NaiveDateTime,
    pub period_end: NaiveDateTime,
    pub computed_at: DateTime<Utc>,
}

/// Session statistics for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub date: String,
    pub focus_minutes: u32,
    pub breaks_completed: u32,
    pub sessions_completed: u32,
    pub evasion_attempts: u32,
}

/// Work schedule model for managing work hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSchedule {
    pub id: i32,
    pub user_id: i32,
    pub use_work_schedule: bool,
    pub work_start_time: Option<String>, // "09:00"
    pub work_end_time: Option<String>,   // "18:00"
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for WorkSchedule {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: 1,
            user_id: 1,
            use_work_schedule: false,
            work_start_time: None,
            work_end_time: None,
            timezone: "local".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Database row conversion helpers
impl UserSettings {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            focus_duration: row.get("focus_duration")?,
            short_break_duration: row.get("short_break_duration")?,
            long_break_duration: row.get("long_break_duration")?,
            cycles_per_long_break: row.get("cycles_per_long_break")?,
            pre_alert_seconds: row.get("pre_alert_seconds")?,
            strict_mode: row.get("strict_mode")?,
            pin_hash: row.get("pin_hash")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl BlockListItem {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let type_str: String = row.get("type")?;
        let item_type = type_str.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?;

        Ok(Self {
            id: Some(row.get("id")?),
            item_type,
            value: row.get("value")?,
            platform: row.get("platform")?,
            enabled: row.get("enabled")?,
            created_at: row.get("created_at")?,
        })
    }
}

impl Session {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let type_str: String = row.get("session_type")?;
        let session_type = type_str.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?;

        Ok(Self {
            id: row.get("id")?,
            session_type,
            start_time: row.get("start_time")?,
            end_time: row.get("end_time")?,
            planned_duration: row.get("planned_duration")?,
            actual_duration: row.get("actual_duration")?,
            strict_mode: row.get("strict_mode")?,
            completed: row.get("completed")?,
            notes: row.get("notes")?,
            created_at: row.get("created_at")?,
        })
    }
}

impl EvasionAttempt {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        let type_str: String = row.get("attempt_type")?;
        let attempt_type = type_str.parse().map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            )
        })?;

        Ok(Self {
            id: Some(row.get("id")?),
            session_id: row.get("session_id")?,
            attempt_type,
            blocked_item: row.get("blocked_item")?,
            timestamp: row.get("timestamp")?,
        })
    }
}

impl Insight {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: Some(row.get("id")?),
            metric_key: row.get("metric_key")?,
            metric_value: row.get("metric_value")?,
            period_start: row.get("period_start")?,
            period_end: row.get("period_end")?,
            computed_at: row.get("computed_at")?,
        })
    }
}

impl WorkSchedule {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            use_work_schedule: row.get("use_work_schedule")?,
            work_start_time: row.get("work_start_time")?,
            work_end_time: row.get("work_end_time")?,
            timezone: row.get("timezone")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
