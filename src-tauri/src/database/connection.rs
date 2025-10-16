use crate::database::migrations::MigrationManager;
use crate::database::models::{Session, SessionType, UserSettings};
use crate::database::{DatabaseError, DatabaseResult};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database connection manager with connection pooling
pub struct DatabaseManager {
    connection: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DatabaseManager {
    /// Create a new database manager
    pub fn new(db_path: PathBuf) -> DatabaseResult<Self> {
        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DatabaseError::ConnectionPool(format!("Failed to create database directory: {}", e))
            })?;
        }

        // Open connection with appropriate flags
        let connection = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(DatabaseError::Sqlite)?;

        // Configure SQLite settings for optimal performance
        Self::configure_connection(&connection)?;

        let manager = DatabaseManager {
            connection: Arc::new(Mutex::new(connection)),
            db_path,
        };

        // Initialize or migrate database
        manager.initialize_or_migrate()?;

        Ok(manager)
    }

    /// Get a reference to the database connection
    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.connection)
    }

    /// Execute a function with the database connection
    pub fn with_connection<F, R>(&self, f: F) -> DatabaseResult<R>
    where
        F: FnOnce(&Connection) -> DatabaseResult<R>,
    {
        let conn = self.connection.lock().map_err(|e| {
            DatabaseError::ConnectionPool(format!("Failed to acquire connection lock: {}", e))
        })?;

        f(&*conn)
    }

    /// Initialize database or run migrations
    fn initialize_or_migrate(&self) -> DatabaseResult<()> {
        self.with_connection(|conn| {
            // Check if database exists
            if !MigrationManager::database_exists(conn)? {
                println!("Initializing new database at: {:?}", self.db_path);
                MigrationManager::initialize_database(conn)?;
            } else {
                println!("Database found, checking for migrations...");
                MigrationManager::migrate_to_current(conn)?;
            }

            // Validate database integrity
            MigrationManager::validate_database(conn)?;

            Ok(())
        })
    }

    /// Configure SQLite connection settings
    fn configure_connection(conn: &Connection) -> DatabaseResult<()> {
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(DatabaseError::Sqlite)?;

        // Set WAL mode for better concurrency
        let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?;

        // Set synchronous mode for better performance
        let _: String = conn.query_row("PRAGMA synchronous = NORMAL", [], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?;

        // Set cache size (negative value means KB)
        let _: i32 = conn.query_row("PRAGMA cache_size = -64000", [], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?;

        // Set temp store to memory
        let _: String = conn.query_row("PRAGMA temp_store = MEMORY", [], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?;

        // Set mmap size for better I/O performance
        let _: i64 = conn.query_row("PRAGMA mmap_size = 268435456", [], |row| row.get(0))
            .map_err(DatabaseError::Sqlite)?;

        println!("Database connection configured");
        Ok(())
    }

    /// Get database file path
    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Create a backup of the database
    pub fn backup(&self, backup_path: &str) -> DatabaseResult<()> {
        let source_path = self.db_path.to_string_lossy();
        MigrationManager::backup_database(&source_path, backup_path)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> DatabaseResult<DatabaseStats> {
        self.with_connection(|conn| {
            let page_count: i64 = conn
                .query_row("PRAGMA page_count", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;

            let page_size: i64 = conn
                .query_row("PRAGMA page_size", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;

            let freelist_count: i64 = conn
                .query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;

            let user_version: i32 = conn
                .query_row("PRAGMA user_version", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;

            Ok(DatabaseStats {
                page_count,
                page_size,
                freelist_count,
                total_size: page_count * page_size,
                free_size: freelist_count * page_size,
                user_version,
            })
        })
    }

    /// User Settings Methods

    /// Get user settings
    pub fn get_user_settings(&self) -> DatabaseResult<Option<UserSettings>> {
        self.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, focus_duration, short_break_duration, long_break_duration, 
                        cycles_per_long_break, pre_alert_seconds, strict_mode, pin_hash, 
                        created_at, updated_at 
                 FROM user_settings 
                 WHERE id = 1",
                )
                .map_err(DatabaseError::Sqlite)?;

            let result = stmt.query_row([], |row| UserSettings::from_row(row));

            match result {
                Ok(settings) => Ok(Some(settings)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DatabaseError::Sqlite(e)),
            }
        })
    }

    /// Save user settings
    pub fn save_user_settings(&self, settings: &UserSettings) -> DatabaseResult<()> {
        self.with_connection(|conn| {
            conn.execute(
                "INSERT OR REPLACE INTO user_settings 
                 (id, focus_duration, short_break_duration, long_break_duration, 
                  cycles_per_long_break, pre_alert_seconds, strict_mode, pin_hash, 
                  created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    settings.id,
                    settings.focus_duration,
                    settings.short_break_duration,
                    settings.long_break_duration,
                    settings.cycles_per_long_break,
                    settings.pre_alert_seconds,
                    settings.strict_mode,
                    settings.pin_hash,
                    settings.created_at,
                    settings.updated_at,
                ],
            )
            .map_err(DatabaseError::Sqlite)?;

            Ok(())
        })
    }

    /// Session Management Methods

    /// Create a new session
    pub fn create_session(&self, session: &Session) -> DatabaseResult<()> {
        self.with_connection(|conn| {
            conn.execute(
                "INSERT INTO sessions 
                 (id, session_type, start_time, end_time, planned_duration, 
                  actual_duration, strict_mode, completed, notes, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    session.id,
                    session.session_type.to_string(),
                    session.start_time,
                    session.end_time,
                    session.planned_duration,
                    session.actual_duration,
                    session.strict_mode,
                    session.completed,
                    session.notes,
                    session.created_at,
                ],
            )
            .map_err(DatabaseError::Sqlite)?;

            Ok(())
        })
    }

    /// Update an existing session
    pub fn update_session(&self, session: &Session) -> DatabaseResult<()> {
        self.with_connection(|conn| {
            conn.execute(
                "UPDATE sessions 
                 SET session_type = ?2, start_time = ?3, end_time = ?4, 
                     planned_duration = ?5, actual_duration = ?6, strict_mode = ?7, 
                     completed = ?8, notes = ?9
                 WHERE id = ?1",
                params![
                    session.id,
                    session.session_type.to_string(),
                    session.start_time,
                    session.end_time,
                    session.planned_duration,
                    session.actual_duration,
                    session.strict_mode,
                    session.completed,
                    session.notes,
                ],
            )
            .map_err(DatabaseError::Sqlite)?;

            Ok(())
        })
    }

    /// Get the most recent active (incomplete) session
    pub fn get_active_session(&self) -> DatabaseResult<Option<Session>> {
        self.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_type, start_time, end_time, planned_duration, 
                        actual_duration, strict_mode, completed, notes, created_at
                 FROM sessions 
                 WHERE completed = FALSE AND end_time IS NULL
                 ORDER BY start_time DESC 
                 LIMIT 1",
                )
                .map_err(DatabaseError::Sqlite)?;

            let result = stmt.query_row([], |row| Session::from_row(row));

            match result {
                Ok(session) => Ok(Some(session)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DatabaseError::Sqlite(e)),
            }
        })
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> DatabaseResult<Option<Session>> {
        self.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_type, start_time, end_time, planned_duration, 
                        actual_duration, strict_mode, completed, notes, created_at
                 FROM sessions 
                 WHERE id = ?1",
                )
                .map_err(DatabaseError::Sqlite)?;

            let result = stmt.query_row([session_id], |row| Session::from_row(row));

            match result {
                Ok(session) => Ok(Some(session)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(DatabaseError::Sqlite(e)),
            }
        })
    }

    /// Get sessions within a date range
    pub fn get_sessions_in_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> DatabaseResult<Vec<Session>> {
        self.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, session_type, start_time, end_time, planned_duration, 
                        actual_duration, strict_mode, completed, notes, created_at
                 FROM sessions 
                 WHERE start_time >= ?1 AND start_time <= ?2
                 ORDER BY start_time ASC",
                )
                .map_err(DatabaseError::Sqlite)?;

            let session_iter = stmt
                .query_map([start_date, end_date], |row| Session::from_row(row))
                .map_err(DatabaseError::Sqlite)?;

            let mut sessions = Vec::new();
            for session in session_iter {
                sessions.push(session.map_err(DatabaseError::Sqlite)?);
            }

            Ok(sessions)
        })
    }

    /// Get session statistics for the last N days
    pub fn get_session_stats(
        &self,
        days: u32,
    ) -> DatabaseResult<Vec<crate::database::models::SessionStats>> {
        use chrono::{Duration, NaiveDate};

        self.with_connection(|conn| {
            let end_date = Utc::now();
            let start_date = end_date - Duration::days(days as i64);

            let mut stmt = conn
                .prepare(
                    "SELECT 
                    DATE(start_time) as date,
                    SUM(CASE 
                        WHEN session_type = 'focus' AND completed = 1 
                        THEN COALESCE(actual_duration, 0) / 60 
                        ELSE 0 
                    END) as focus_minutes,
                    COUNT(CASE 
                        WHEN session_type IN ('short_break', 'long_break') AND completed = 1 
                        THEN 1 
                    END) as breaks_completed,
                    COUNT(CASE 
                        WHEN session_type = 'focus' AND completed = 1 
                        THEN 1 
                    END) as sessions_completed,
                    COUNT(CASE 
                        WHEN session_type = 'focus' 
                        THEN (SELECT COUNT(*) FROM evasion_attempts WHERE session_id = sessions.id)
                    END) as evasion_attempts
                 FROM sessions 
                 WHERE start_time >= ?1 AND start_time <= ?2
                 GROUP BY DATE(start_time)
                 ORDER BY date DESC",
                )
                .map_err(DatabaseError::Sqlite)?;

            let stats_iter = stmt
                .query_map([start_date, end_date], |row| {
                    Ok(crate::database::models::SessionStats {
                        date: row.get::<_, String>("date")?,
                        focus_minutes: row.get::<_, i64>("focus_minutes")? as u32,
                        breaks_completed: row.get::<_, i64>("breaks_completed")? as u32,
                        sessions_completed: row.get::<_, i64>("sessions_completed")? as u32,
                        evasion_attempts: row.get::<_, i64>("evasion_attempts")? as u32,
                    })
                })
                .map_err(DatabaseError::Sqlite)?;

            let mut stats = Vec::new();
            for stat in stats_iter {
                stats.push(stat.map_err(DatabaseError::Sqlite)?);
            }

            Ok(stats)
        })
    }
}

/// Database statistics structure
#[derive(Debug)]
pub struct DatabaseStats {
    pub page_count: i64,
    pub page_size: i64,
    pub freelist_count: i64,
    pub total_size: i64,
    pub free_size: i64,
    pub user_version: i32,
}

impl DatabaseStats {
    pub fn used_size(&self) -> i64 {
        self.total_size - self.free_size
    }

    pub fn usage_percentage(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.used_size() as f64 / self.total_size as f64) * 100.0
        }
    }
}
