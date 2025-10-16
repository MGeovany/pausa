#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{DatabaseManager, models::*};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_db() -> DatabaseManager {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        DatabaseManager::new(db_path).unwrap()
    }

    #[test]
    fn test_database_initialization() {
        let db = create_test_db();
        
        // Test that we can get stats from the initialized database
        let stats = db.get_stats().unwrap();
        assert!(stats.page_count > 0);
        assert!(stats.page_size > 0);
    }

    #[test]
    fn test_database_validation() {
        let db = create_test_db();
        
        // Test database validation passes
        db.with_connection(|conn| {
            crate::database::migrations::MigrationManager::validate_database(conn)
        }).unwrap();
    }

    #[test]
    fn test_schema_version() {
        let db = create_test_db();
        
        // Test that schema version is correctly set
        db.with_connection(|conn| {
            let version = crate::database::migrations::MigrationManager::get_current_version(conn)?;
            assert_eq!(version, crate::database::schema::SCHEMA_VERSION);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_default_user_settings() {
        let db = create_test_db();
        
        // Test that default user settings are inserted
        db.with_connection(|conn| {
            let settings: UserSettings = conn.query_row(
                "SELECT * FROM user_settings WHERE id = 1",
                [],
                |row| UserSettings::from_row(row)
            ).map_err(crate::database::DatabaseError::Sqlite)?;
            
            assert_eq!(settings.id, 1);
            assert_eq!(settings.focus_duration, 1500);
            assert_eq!(settings.short_break_duration, 300);
            assert_eq!(settings.long_break_duration, 900);
            assert_eq!(settings.cycles_per_long_break, 4);
            assert_eq!(settings.pre_alert_seconds, 120);
            assert!(!settings.strict_mode);
            assert!(settings.pin_hash.is_none());
            
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_foreign_key_constraints() {
        let db = create_test_db();
        
        // Test that foreign key constraints are enabled
        db.with_connection(|conn| {
            let fk_enabled: bool = conn.query_row(
                "PRAGMA foreign_keys",
                [],
                |row| row.get(0)
            ).map_err(crate::database::DatabaseError::Sqlite)?;
            
            assert!(fk_enabled);
            Ok(())
        }).unwrap();
    }

    #[test]
    fn test_table_existence() {
        let db = create_test_db();
        
        let required_tables = vec![
            "user_settings",
            "block_list",
            "sessions", 
            "evasion_attempts",
            "insights",
            "schema_version"
        ];
        
        db.with_connection(|conn| {
            for table in required_tables {
                let exists: bool = conn.query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?",
                    [table],
                    |row| row.get(0)
                ).map_err(crate::database::DatabaseError::Sqlite)?;
                
                assert!(exists, "Table '{}' should exist", table);
            }
            Ok(())
        }).unwrap();
    }
}
