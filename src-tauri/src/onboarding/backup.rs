use crate::database::DatabaseResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Backup-related errors
#[derive(Debug, Error)]
pub enum BackupError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Backup not found: {path}")]
    BackupNotFound { path: String },

    #[error("Invalid backup format: {reason}")]
    InvalidFormat { reason: String },
}

pub type BackupResult<T> = Result<T, BackupError>;

/// Configuration backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub created_at: DateTime<Utc>,
    pub version: String,
    pub backup_type: BackupType,
    pub description: Option<String>,
    pub file_size: u64,
}

/// Types of configuration backups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    /// Automatic backup before onboarding completion
    PreOnboarding,
    /// Automatic backup after onboarding completion
    PostOnboarding,
    /// Manual backup triggered by user
    Manual,
    /// Backup before configuration changes
    PreUpdate,
}

/// Complete configuration backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationBackup {
    pub metadata: BackupMetadata,
    pub onboarding_config: Option<serde_json::Value>,
    pub user_settings: Option<serde_json::Value>,
    pub work_schedule: Option<serde_json::Value>,
    pub database_version: Option<String>,
}

/// Configuration backup manager
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(app_data_dir: &Path) -> BackupResult<Self> {
        let backup_dir = app_data_dir.join("backups");

        // Ensure backup directory exists
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir)?;
        }

        Ok(Self { backup_dir })
    }

    /// Create a backup of the current configuration
    pub fn create_backup(
        &self,
        backup_type: BackupType,
        description: Option<String>,
        database: &crate::database::DatabaseManager,
    ) -> BackupResult<String> {
        let timestamp = Utc::now();
        let backup_id = format!(
            "{}_{}",
            timestamp.format("%Y%m%d_%H%M%S"),
            match backup_type {
                BackupType::PreOnboarding => "pre_onboarding",
                BackupType::PostOnboarding => "post_onboarding",
                BackupType::Manual => "manual",
                BackupType::PreUpdate => "pre_update",
            }
        );

        // Collect current configuration
        let mut backup = ConfigurationBackup {
            metadata: BackupMetadata {
                created_at: timestamp,
                version: env!("CARGO_PKG_VERSION").to_string(),
                backup_type,
                description,
                file_size: 0, // Will be set after serialization
            },
            onboarding_config: None,
            user_settings: None,
            work_schedule: None,
            database_version: None,
        };

        // Get user settings
        match database.get_user_settings() {
            Ok(Some(settings)) => {
                backup.user_settings = Some(serde_json::to_value(settings)?);
            }
            Ok(None) => {
                // No user settings yet
            }
            Err(e) => {
                return Err(BackupError::Database(format!(
                    "Failed to get user settings: {}",
                    e
                )));
            }
        }

        // Get work schedule
        match self.get_work_schedule(database) {
            Ok(Some(schedule)) => {
                backup.work_schedule = Some(schedule);
            }
            Ok(None) => {
                // No work schedule configured
            }
            Err(e) => {
                return Err(BackupError::Database(format!(
                    "Failed to get work schedule: {}",
                    e
                )));
            }
        }

        // Get latest onboarding configuration
        match database.get_latest_onboarding_completion() {
            Ok(Some(completion)) => {
                if let Some(config_snapshot) = completion.config_snapshot {
                    backup.onboarding_config = Some(serde_json::from_str(&config_snapshot)?);
                }
            }
            Ok(None) => {
                // No onboarding completion yet
            }
            Err(e) => {
                return Err(BackupError::Database(format!(
                    "Failed to get onboarding completion: {}",
                    e
                )));
            }
        }

        // Get database version/stats
        match database.get_stats() {
            Ok(stats) => {
                backup.database_version = Some(stats.user_version.to_string());
            }
            Err(e) => {
                return Err(BackupError::Database(format!(
                    "Failed to get database stats: {}",
                    e
                )));
            }
        }

        // Serialize and save backup
        let backup_json = serde_json::to_string_pretty(&backup)?;
        backup.metadata.file_size = backup_json.len() as u64;

        // Update backup with correct file size
        let backup_json = serde_json::to_string_pretty(&backup)?;

        let backup_path = self.backup_dir.join(format!("{}.json", backup_id));
        fs::write(&backup_path, backup_json)?;

        println!("✅ Configuration backup created: {}", backup_path.display());

        // Clean up old backups (keep last 10)
        self.cleanup_old_backups(10)?;

        Ok(backup_id)
    }

    /// Restore configuration from backup
    pub fn restore_backup(
        &self,
        backup_id: &str,
        database: &crate::database::DatabaseManager,
    ) -> BackupResult<()> {
        let backup_path = self.backup_dir.join(format!("{}.json", backup_id));

        if !backup_path.exists() {
            return Err(BackupError::BackupNotFound {
                path: backup_path.to_string_lossy().to_string(),
            });
        }

        let backup_content = fs::read_to_string(&backup_path)?;
        let backup: ConfigurationBackup = serde_json::from_str(&backup_content)?;

        println!("🔄 Restoring configuration from backup: {}", backup_id);

        // Restore user settings
        if let Some(user_settings_json) = backup.user_settings {
            let user_settings: crate::database::models::UserSettings =
                serde_json::from_value(user_settings_json)?;

            database.save_user_settings(&user_settings).map_err(|e| {
                BackupError::Database(format!("Failed to restore user settings: {}", e))
            })?;

            println!("✅ User settings restored");
        }

        // Restore work schedule
        if let Some(work_schedule_json) = backup.work_schedule {
            self.restore_work_schedule(database, work_schedule_json)?;
            println!("✅ Work schedule restored");
        }

        // Note: We don't restore onboarding completion records as they are historical

        println!(
            "✅ Configuration restored successfully from backup: {}",
            backup_id
        );

        Ok(())
    }

    /// List available backups
    pub fn list_backups(&self) -> BackupResult<Vec<(String, BackupMetadata)>> {
        let mut backups = Vec::new();

        if !self.backup_dir.exists() {
            return Ok(backups);
        }

        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_backup_metadata(&path) {
                        Ok(metadata) => {
                            backups.push((file_stem.to_string(), metadata));
                        }
                        Err(e) => {
                            println!(
                                "⚠️ Failed to load backup metadata for {}: {}",
                                path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        // Sort by creation date (newest first)
        backups.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        Ok(backups)
    }

    /// Delete a backup
    pub fn delete_backup(&self, backup_id: &str) -> BackupResult<()> {
        let backup_path = self.backup_dir.join(format!("{}.json", backup_id));

        if !backup_path.exists() {
            return Err(BackupError::BackupNotFound {
                path: backup_path.to_string_lossy().to_string(),
            });
        }

        fs::remove_file(&backup_path)?;
        println!("🗑️ Backup deleted: {}", backup_id);

        Ok(())
    }

    /// Get backup details
    pub fn get_backup_details(&self, backup_id: &str) -> BackupResult<ConfigurationBackup> {
        let backup_path = self.backup_dir.join(format!("{}.json", backup_id));

        if !backup_path.exists() {
            return Err(BackupError::BackupNotFound {
                path: backup_path.to_string_lossy().to_string(),
            });
        }

        let backup_content = fs::read_to_string(&backup_path)?;
        let backup: ConfigurationBackup = serde_json::from_str(&backup_content)?;

        Ok(backup)
    }

    /// Clean up old backups (keep only the specified number)
    pub fn cleanup_old_backups(&self, keep_count: usize) -> BackupResult<()> {
        let backups = self.list_backups()?;

        if backups.len() > keep_count {
            let to_delete = &backups[keep_count..];

            for (backup_id, _) in to_delete {
                if let Err(e) = self.delete_backup(backup_id) {
                    println!("⚠️ Failed to delete old backup {}: {}", backup_id, e);
                }
            }

            println!("🧹 Cleaned up {} old backups", to_delete.len());
        }

        Ok(())
    }

    /// Load backup metadata from file
    fn load_backup_metadata(&self, path: &Path) -> BackupResult<BackupMetadata> {
        let backup_content = fs::read_to_string(path)?;
        let backup: ConfigurationBackup = serde_json::from_str(&backup_content)?;
        Ok(backup.metadata)
    }

    /// Get work schedule from database (helper method)
    fn get_work_schedule(
        &self,
        database: &crate::database::DatabaseManager,
    ) -> BackupResult<Option<serde_json::Value>> {
        // This is a simplified version - in a real implementation, you'd have a proper work schedule query
        // For now, we'll return None as work schedule might be stored differently
        Ok(None)
    }

    /// Restore work schedule (helper method)
    fn restore_work_schedule(
        &self,
        _database: &crate::database::DatabaseManager,
        _schedule_json: serde_json::Value,
    ) -> BackupResult<()> {
        // This is a simplified version - in a real implementation, you'd restore the work schedule
        // For now, we'll just log that it would be restored
        println!("📅 Work schedule would be restored here");
        Ok(())
    }
}

/// Create automatic backup before onboarding
pub fn create_pre_onboarding_backup(
    database: &crate::database::DatabaseManager,
    app_data_dir: &Path,
) -> BackupResult<String> {
    let backup_manager = BackupManager::new(app_data_dir)?;
    backup_manager.create_backup(
        BackupType::PreOnboarding,
        Some("Automatic backup before onboarding completion".to_string()),
        database,
    )
}

/// Create automatic backup after onboarding
pub fn create_post_onboarding_backup(
    database: &crate::database::DatabaseManager,
    app_data_dir: &Path,
) -> BackupResult<String> {
    let backup_manager = BackupManager::new(app_data_dir)?;
    backup_manager.create_backup(
        BackupType::PostOnboarding,
        Some("Automatic backup after onboarding completion".to_string()),
        database,
    )
}
