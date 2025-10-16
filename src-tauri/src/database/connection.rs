use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use rusqlite::{Connection, OpenFlags};
use crate::database::{DatabaseError, DatabaseResult};
use crate::database::migrations::MigrationManager;

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
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::ConnectionPool(
                    format!("Failed to create database directory: {}", e)
                ))?;
        }
        
        // Open connection with appropriate flags
        let connection = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE 
                | OpenFlags::SQLITE_OPEN_CREATE 
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
        ).map_err(DatabaseError::Sqlite)?;
        
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
        let conn = self.connection.lock()
            .map_err(|e| DatabaseError::ConnectionPool(
                format!("Failed to acquire connection lock: {}", e)
            ))?;
        
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
        conn.execute("PRAGMA journal_mode = WAL", [])
            .map_err(DatabaseError::Sqlite)?;
        
        // Set synchronous mode for better performance
        conn.execute("PRAGMA synchronous = NORMAL", [])
            .map_err(DatabaseError::Sqlite)?;
        
        // Set cache size (negative value means KB)
        conn.execute("PRAGMA cache_size = -64000", []) // 64MB cache
            .map_err(DatabaseError::Sqlite)?;
        
        // Set temp store to memory
        conn.execute("PRAGMA temp_store = MEMORY", [])
            .map_err(DatabaseError::Sqlite)?;
        
        // Set mmap size for better I/O performance
        conn.execute("PRAGMA mmap_size = 268435456", []) // 256MB
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
            let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;
            
            let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;
            
            let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .map_err(DatabaseError::Sqlite)?;
            
            let user_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))
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
