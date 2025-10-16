pub mod connection;
pub mod migrations;
pub mod models;
pub mod schema;

#[cfg(test)]
mod tests;

pub use connection::DatabaseManager;
pub use models::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Connection pool error: {0}")]
    ConnectionPool(String),

    #[error("Data validation error: {0}")]
    Validation(String),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
