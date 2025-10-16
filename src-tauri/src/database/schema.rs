/// Database schema definitions for Pausa application
/// Based on the design document specifications

pub const SCHEMA_VERSION: i32 = 1;

/// Initial database schema - creates all tables
pub const INITIAL_SCHEMA: &str = r#"
-- User configuration table
CREATE TABLE user_settings (
    id INTEGER PRIMARY KEY,
    focus_duration INTEGER NOT NULL DEFAULT 1500, -- 25 minutes in seconds
    short_break_duration INTEGER NOT NULL DEFAULT 300, -- 5 minutes
    long_break_duration INTEGER NOT NULL DEFAULT 900, -- 15 minutes
    cycles_per_long_break INTEGER NOT NULL DEFAULT 4,
    pre_alert_seconds INTEGER NOT NULL DEFAULT 120,
    strict_mode BOOLEAN NOT NULL DEFAULT FALSE,
    pin_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Blocked applications and websites
CREATE TABLE block_list (
    id INTEGER PRIMARY KEY,
    type TEXT NOT NULL CHECK (type IN ('app', 'website')),
    value TEXT NOT NULL,
    platform TEXT, -- 'windows', 'macos', 'linux', or NULL for all
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Focus and break sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    session_type TEXT NOT NULL CHECK (session_type IN ('focus', 'short_break', 'long_break')),
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    planned_duration INTEGER NOT NULL, -- seconds
    actual_duration INTEGER, -- seconds, NULL if not completed
    strict_mode BOOLEAN NOT NULL DEFAULT FALSE,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Evasion attempts (when user tries to access blocked content)
CREATE TABLE evasion_attempts (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    attempt_type TEXT NOT NULL CHECK (attempt_type IN ('app', 'website')),
    blocked_item TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions (id)
);

-- Computed insights and statistics
CREATE TABLE insights (
    id INTEGER PRIMARY KEY,
    metric_key TEXT NOT NULL,
    metric_value REAL NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    computed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Schema version tracking
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Performance indexes
CREATE INDEX idx_sessions_start_time ON sessions (start_time);
CREATE INDEX idx_sessions_type ON sessions (session_type);
CREATE INDEX idx_sessions_completed ON sessions (completed);
CREATE INDEX idx_block_list_type_value ON block_list (type, value);
CREATE INDEX idx_block_list_enabled ON block_list (enabled);
CREATE INDEX idx_evasion_attempts_session ON evasion_attempts (session_id);
CREATE INDEX idx_evasion_attempts_timestamp ON evasion_attempts (timestamp);
CREATE INDEX idx_insights_key_period ON insights (metric_key, period_start, period_end);

-- Insert initial schema version
INSERT INTO schema_version (version) VALUES (1);

-- Insert default user settings
INSERT INTO user_settings (id) VALUES (1);
"#;

/// SQL statements for creating individual tables (used in migrations)
pub const CREATE_USER_SETTINGS: &str = r#"
CREATE TABLE user_settings (
    id INTEGER PRIMARY KEY,
    focus_duration INTEGER NOT NULL DEFAULT 1500,
    short_break_duration INTEGER NOT NULL DEFAULT 300,
    long_break_duration INTEGER NOT NULL DEFAULT 900,
    cycles_per_long_break INTEGER NOT NULL DEFAULT 4,
    pre_alert_seconds INTEGER NOT NULL DEFAULT 120,
    strict_mode BOOLEAN NOT NULL DEFAULT FALSE,
    pin_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
)
"#;

pub const CREATE_BLOCK_LIST: &str = r#"
CREATE TABLE block_list (
    id INTEGER PRIMARY KEY,
    type TEXT NOT NULL CHECK (type IN ('app', 'website')),
    value TEXT NOT NULL,
    platform TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
)
"#;

pub const CREATE_SESSIONS: &str = r#"
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    session_type TEXT NOT NULL CHECK (session_type IN ('focus', 'short_break', 'long_break')),
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    planned_duration INTEGER NOT NULL,
    actual_duration INTEGER,
    strict_mode BOOLEAN NOT NULL DEFAULT FALSE,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
)
"#;

pub const CREATE_EVASION_ATTEMPTS: &str = r#"
CREATE TABLE evasion_attempts (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    attempt_type TEXT NOT NULL CHECK (attempt_type IN ('app', 'website')),
    blocked_item TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions (id)
)
"#;

pub const CREATE_INSIGHTS: &str = r#"
CREATE TABLE insights (
    id INTEGER PRIMARY KEY,
    metric_key TEXT NOT NULL,
    metric_value REAL NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    computed_at DATETIME DEFAULT CURRENT_TIMESTAMP
)
"#;

pub const CREATE_SCHEMA_VERSION: &str = r#"
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
)
"#;
