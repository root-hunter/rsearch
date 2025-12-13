use rusqlite::{Connection, Result};

const DATABASE_FILE: &str = "storage.db";

struct StorageEngine {
    conn: Connection,
}

impl StorageEngine {
    pub fn new() -> Self {
        let conn = Connection::open(DATABASE_FILE).expect("Failed to open database");
        StorageEngine { conn }
    }

    pub fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }
}