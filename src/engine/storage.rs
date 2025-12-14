use std::env;

use once_cell::sync::Lazy;
use rusqlite::{Connection, Result};

pub static STORAGE_DATABASE_PATH: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("DATABASE_FILE")
            .unwrap_or_else(|_| "storage.db".into())
            .into_boxed_str(),
    )
});


#[derive(Debug)]
pub enum StorageError {
    InitializationError(rusqlite::Error),
    ExecutionError(rusqlite::Error),
}

#[derive(Debug)]
pub struct StorageEngine;

impl StorageEngine {
    pub fn new() -> Self {
        StorageEngine
    }

    pub fn initialize(conn: &Connection) -> Result<(), StorageError> {
        conn
            .pragma_update(None, "journal_mode", &"WAL")
            .map_err(StorageError::InitializationError)?;

        conn
            .pragma_update(None, "cache_size", &"-2000")
            .map_err(StorageError::InitializationError)?;
        conn
            .pragma_update(None, "temp_store", &"MEMORY")
            .map_err(StorageError::InitializationError)?;
        conn
            .pragma_update(None, "locking_mode", &"EXCLUSIVE")
            .map_err(StorageError::InitializationError)?;

        conn
            .execute(
                "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                extension TEXT,
                status TEXT NOT NULL DEFAULT 'New'
            )",
                [],
            )
            .map_err(StorageError::InitializationError)?;

        conn
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_documents_path ON documents(path)",
                [],
            )
            .map_err(StorageError::InitializationError)?;

        conn
            .execute(
                "CREATE VIRTUAL TABLE IF NOT EXISTS index_documents USING fts5 (
                document_id UNINDEXED,
                content,
                description
            )",
                [],
            )
            .map_err(StorageError::InitializationError)?;
        Ok(())
    }
}
