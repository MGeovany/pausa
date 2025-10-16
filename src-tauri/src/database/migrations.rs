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
            _ => Err(DatabaseError::Migration(format!(
                "Unknown migration version: {}",
                version
            ))),
        }
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
}
