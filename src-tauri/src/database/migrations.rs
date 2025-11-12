use crate::database::schema::{INITIAL_SCHEMA, SCHEMA_VERSION};
use crate::database::{DatabaseError, DatabaseResult};
use rusqlite::{Connection, OptionalExtension, Result as SqliteResult};

/// Migration manager for handling database schema changes
pub struct MigrationManager;

impl MigrationManager {
    /// Initialize a new database with the current schema
    pub fn initialize_database(conn: &Connection) -> DatabaseResult<()> {
        // Execute the initial schema
        conn.execute_batch(INITIAL_SCHEMA)
            .map_err(DatabaseError::Sqlite)?;

        println!(
            "Database initialized with schema version {}",
            SCHEMA_VERSION
        );
        Ok(())
    }

    /// Check if database exists and has tables
    pub fn database_exists(conn: &Connection) -> DatabaseResult<bool> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='schema_version'")
            .map_err(DatabaseError::Sqlite)?;

        let exists = stmt.exists([]).map_err(DatabaseError::Sqlite)?;
        Ok(exists)
    }

    /// Get current database schema version
    pub fn get_current_version(conn: &Connection) -> DatabaseResult<i32> {
        let version: Option<i32> = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(DatabaseError::Sqlite)?;

        Ok(version.unwrap_or(0))
    }

    /// Run migrations to bring database up to current version
    pub fn migrate_to_current(conn: &Connection) -> DatabaseResult<()> {
        let current_version = Self::get_current_version(conn)?;

        if current_version < SCHEMA_VERSION {
            println!(
                "Migrating database from version {} to {}",
                current_version, SCHEMA_VERSION
            );

            // Apply migrations in sequence
            for version in (current_version + 1)..=SCHEMA_VERSION {
                Self::apply_migration(conn, version)?;
            }

            println!("Database migration completed");
        } else if current_version > SCHEMA_VERSION {
            return Err(DatabaseError::Migration(format!(
                "Database version {} is newer than application version {}",
                current_version, SCHEMA_VERSION
            )));
        }

        Ok(())
    }

    /// Apply a specific migration version
    fn apply_migration(conn: &Connection, version: i32) -> DatabaseResult<()> {
        match version {
            1 => {
                // Version 1 is the initial schema, already handled in initialize_database
                Ok(())
            }
            2 => {
                // Version 2: Add work_schedule table
                Self::migrate_to_v2(conn)
            }
            3 => {
                // Version 3: Add cycle configuration fields to user_settings
                Self::migrate_to_v3(conn)
            }
            4 => {
                // Version 4: Add onboarding_completion table
                Self::migrate_to_v4(conn)
            }
            5 => {
                // Version 5: Add user_email to onboarding_completion table
                Self::migrate_to_v5(conn)
            }
            6 => {
                // Version 6: Add notification_history table
                Self::migrate_to_v6(conn)
            }
            7 => {
                // Version 7: Add bypass_attempts table
                Self::migrate_to_v7(conn)
            }
            8 => {
                // Version 8: Add within_work_hours and cycle_number to sessions table
                Self::migrate_to_v8(conn)
            }
            _ => Err(DatabaseError::Migration(format!(
                "Unknown migration version: {}",
                version
            ))),
        }
    }

    /// Migration to version 2: Add work_schedule table
    fn migrate_to_v2(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 2: Adding work_schedule table");

        // Create work_schedule table
        conn.execute(
            r#"
            CREATE TABLE work_schedule (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL DEFAULT 1,
                use_work_schedule BOOLEAN NOT NULL DEFAULT FALSE,
                work_start_time TEXT,
                work_end_time TEXT,
                timezone TEXT NOT NULL DEFAULT 'local',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES user_settings (id)
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Insert default work schedule for existing user
        conn.execute("INSERT INTO work_schedule (id, user_id) VALUES (1, 1)", [])
            .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 2 completed successfully");
        Ok(())
    }

    /// Migration to version 3: Add cycle configuration fields to user_settings
    fn migrate_to_v3(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 3: Adding cycle configuration fields");

        // Add new fields to user_settings table
        conn.execute(
            "ALTER TABLE user_settings ADD COLUMN cycles_per_long_break_v2 INTEGER NOT NULL DEFAULT 4",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        conn.execute("ALTER TABLE user_settings ADD COLUMN user_name TEXT", [])
            .map_err(DatabaseError::Sqlite)?;

        conn.execute(
            "ALTER TABLE user_settings ADD COLUMN emergency_key_combination TEXT",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (3)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 3 completed successfully");
        Ok(())
    }

    /// Migration to version 4: Add onboarding_completion table
    fn migrate_to_v4(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 4: Adding onboarding_completion table");

        // Create onboarding_completion table
        conn.execute(
            r#"
            CREATE TABLE onboarding_completion (
                id INTEGER PRIMARY KEY,
                completed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                version TEXT NOT NULL DEFAULT '1.0',
                config_snapshot TEXT
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (4)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 4 completed successfully");
        Ok(())
    }

    /// Migration to version 5: Add user_email to onboarding_completion for user-specific tracking
    fn migrate_to_v5(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 5: Adding user_email to onboarding_completion");

        // Add user_email column to onboarding_completion table
        conn.execute(
            "ALTER TABLE onboarding_completion ADD COLUMN user_email TEXT",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (5)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 5 completed successfully");
        Ok(())
    }

    /// Validate database integrity
    pub fn validate_database(conn: &Connection) -> DatabaseResult<()> {
        // Check that all required tables exist
        let required_tables = vec![
            "user_settings",
            "block_list",
            "sessions",
            "evasion_attempts",
            "insights",
            "work_schedule",
            "onboarding_completion",
            "notification_history",
            "bypass_attempts",
            "schema_version",
        ];

        for table in required_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?",
                    [table],
                    |row| row.get(0),
                )
                .map_err(DatabaseError::Sqlite)?;

            if !exists {
                return Err(DatabaseError::Migration(format!(
                    "Required table '{}' is missing",
                    table
                )));
            }
        }

        // Run PRAGMA integrity_check
        let mut stmt = conn
            .prepare("PRAGMA integrity_check")
            .map_err(DatabaseError::Sqlite)?;

        let mut integrity_ok = false;
        let mut errors = Vec::new();

        let rows = stmt
            .query_map([], |row| {
                let result: String = row.get(0)?;
                Ok(result)
            })
            .map_err(DatabaseError::Sqlite)?;

        for row_result in rows {
            let result = row_result.map_err(DatabaseError::Sqlite)?;
            if result == "ok" {
                integrity_ok = true;
            } else {
                errors.push(result);
            }
        }

        // Only fail if we got errors but no "ok"
        if !integrity_ok && !errors.is_empty() {
            return Err(DatabaseError::Migration(format!(
                "Database integrity check failed: {}",
                errors.join(", ")
            )));
        }

        // If no rows returned, that's okay for a new database

        println!("Database validation passed");
        Ok(())
    }

    /// Create a backup of the database before migrations
    pub fn backup_database(source_path: &str, backup_path: &str) -> DatabaseResult<()> {
        std::fs::copy(source_path, backup_path).map_err(|e| {
            DatabaseError::Migration(format!("Failed to create database backup: {}", e))
        })?;

        println!("Database backed up to: {}", backup_path);
        Ok(())
    }

    /// Migration to version 6: Add notification_history table
    fn migrate_to_v6(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 6: Adding notification_history table");

        // Create notification_history table
        conn.execute(
            r#"
            CREATE TABLE notification_history (
                id INTEGER PRIMARY KEY,
                session_id TEXT,
                notification_type TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                sent_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions (id)
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX idx_notification_history_sent_at ON notification_history (sent_at)",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        conn.execute(
            "CREATE INDEX idx_notification_history_session ON notification_history (session_id)",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (6)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 6 completed successfully");
        Ok(())
    }

    /// Migration to version 7: Add bypass_attempts table
    fn migrate_to_v7(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 7: Adding bypass_attempts table");

        // Create bypass_attempts table
        conn.execute(
            r#"
            CREATE TABLE bypass_attempts (
                id INTEGER PRIMARY KEY,
                session_id TEXT NOT NULL,
                method TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Create index for faster queries
        conn.execute(
            "CREATE INDEX idx_bypass_attempts_session ON bypass_attempts (session_id)",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        conn.execute(
            "CREATE INDEX idx_bypass_attempts_created_at ON bypass_attempts (created_at)",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (7)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 7 completed successfully");
        Ok(())
    }

    /// Migration to version 8: Add within_work_hours and cycle_number to sessions table
    fn migrate_to_v8(conn: &Connection) -> DatabaseResult<()> {
        println!("Applying migration to version 8: Adding work hours tracking to sessions");

        // Add within_work_hours column to sessions table
        conn.execute(
            "ALTER TABLE sessions ADD COLUMN within_work_hours BOOLEAN DEFAULT TRUE",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Add cycle_number column to sessions table
        conn.execute("ALTER TABLE sessions ADD COLUMN cycle_number INTEGER", [])
            .map_err(DatabaseError::Sqlite)?;

        // Add is_long_break column to sessions table
        conn.execute(
            "ALTER TABLE sessions ADD COLUMN is_long_break BOOLEAN DEFAULT FALSE",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Create index for work hours queries
        conn.execute(
            "CREATE INDEX idx_sessions_within_work_hours ON sessions (within_work_hours)",
            [],
        )
        .map_err(DatabaseError::Sqlite)?;

        // Update schema version
        conn.execute("INSERT INTO schema_version (version) VALUES (8)", [])
            .map_err(DatabaseError::Sqlite)?;

        println!("Migration to version 8 completed successfully");
        Ok(())
    }
}
