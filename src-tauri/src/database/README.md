# Database Module

This module provides SQLite database functionality for the Pausa application, including schema management, migrations, and data models.

## Components

### Schema (`schema.rs`)
- Defines the complete database schema with all tables and indexes
- Contains SQL statements for table creation
- Manages schema versioning

### Migrations (`migrations.rs`)
- Handles database initialization and schema migrations
- Provides database validation and integrity checks
- Supports database backup functionality

### Connection (`connection.rs`)
- Manages SQLite database connections with connection pooling
- Configures SQLite settings for optimal performance
- Provides thread-safe database access

### Models (`models.rs`)
- Defines Rust structs that map to database tables
- Provides serialization/deserialization for API communication
- Includes helper methods for database row conversion

## Usage

```rust
use crate::database::DatabaseManager;

// Initialize database
let db_path = PathBuf::from("pausa.db");
let db_manager = DatabaseManager::new(db_path)?;

// Use database connection
db_manager.with_connection(|conn| {
    // Perform database operations
    Ok(())
})?;
```

## Database Schema

The database includes the following tables:

- `user_settings`: Application configuration and user preferences
- `block_list`: Blocked applications and websites
- `sessions`: Focus and break session records
- `evasion_attempts`: Log of blocked content access attempts
- `insights`: Computed statistics and metrics
- `schema_version`: Database version tracking

## Performance Optimizations

- WAL mode for better concurrency
- Memory-mapped I/O for improved performance
- Optimized cache size (64MB)
- Strategic indexes for common queries
- Foreign key constraints enabled

## Testing

Run tests with:
```bash
cargo test database
```

Tests cover:
- Database initialization
- Schema validation
- Migration functionality
- Data model conversions
- Foreign key constraints
