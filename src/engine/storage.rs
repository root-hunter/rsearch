use rusqlite::{Connection, Result};

const DATABASE_FILE: &str = "storage.db";

#[derive(Debug)]
pub enum StorageError {
    InitializationError(rusqlite::Error),
    ExecutionError(rusqlite::Error),
}

#[derive(Debug)]
pub struct StorageEngine {
    conn: Connection,
}

impl StorageEngine {
    pub fn new() -> Self {
        let conn = Connection::open(DATABASE_FILE).expect("Failed to open database");
        StorageEngine { conn }
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    pub fn initialize(&self) -> Result<(), StorageError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                extension TEXT NOT NULL
            )",
            [],
        ).map_err(StorageError::InitializationError)?;

        self.conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_documents_path ON documents(path)",
            [],
        ).map_err(StorageError::InitializationError)?;

        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS index_documents USING fts5 (
                document_id UNINDEXED,
                content,
                description
            )",
            [],
        ).map_err(StorageError::InitializationError)?;
        Ok(())
    }
}