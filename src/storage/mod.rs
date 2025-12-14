pub mod commands;

use std::env;

use once_cell::sync::Lazy;
use rusqlite::{Connection, Result};
use tracing::{error, info, warn};

use crate::{
    engine::EngineTask,
    entities::document::{self, Document},
    storage::commands::StorageCommand,
};

const LOG_TARGET: &str = "storage";

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
pub struct StorageEngine {
    channel_tx: crossbeam::channel::Sender<StorageCommand>,
    channel_rx: crossbeam::channel::Receiver<StorageCommand>,
    thread_handle: Option<std::thread::JoinHandle<Result<(), StorageError>>>,
}

impl StorageEngine {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam::channel::unbounded::<StorageCommand>();

        StorageEngine {
            channel_tx: tx,
            channel_rx: rx,
            thread_handle: None,
        }
    }

    pub fn initialize() -> Result<(), StorageError> {
        let conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH)
            .map_err(StorageError::InitializationError)?;

        info!(target: LOG_TARGET, "Initializing storage engine");

        info!(target: LOG_TARGET, "Setting SQLite pragmas");
        conn.pragma_update(None, "journal_mode", &"WAL")
            .map_err(StorageError::InitializationError)?;

        conn.pragma_update(None, "cache_size", &"-2000")
            .map_err(StorageError::InitializationError)?;
        conn.pragma_update(None, "temp_store", &"MEMORY")
            .map_err(StorageError::InitializationError)?;
        conn.pragma_update(None, "locking_mode", &"EXCLUSIVE")
            .map_err(StorageError::InitializationError)?;

        info!(target: LOG_TARGET, "Creating necessary tables and indexes");
        conn.execute(
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

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_documents_path ON documents(path)",
            [],
        )
        .map_err(StorageError::InitializationError)?;

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS index_documents USING fts5 (
                document_id UNINDEXED,
                content,
                description
            )",
            [],
        )
        .map_err(StorageError::InitializationError)?;

        info!(target: LOG_TARGET, "Storage engine initialized successfully");

        Ok(())
    }
}

impl EngineTask<StorageCommand> for StorageEngine {
    fn get_channel_sender(&self) -> &crossbeam::channel::Sender<StorageCommand> {
        &self.channel_tx
    }

    fn get_channel_receiver(&self) -> &crossbeam::channel::Receiver<StorageCommand> {
        &self.channel_rx
    }

    fn run(&mut self) {
        assert!(self.thread_handle.is_none(), "Worker is already running");

        let conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH)
            .map_err(StorageError::InitializationError);

        if let Err(e) = conn {
            error!(target: LOG_TARGET, "Failed to open database connection: {:?}", e);
            return;
        }

        let mut conn = conn.unwrap();

        let receiver = self.channel_rx.clone();

        let handle = std::thread::spawn(move || {
            info!(target: LOG_TARGET, "StorageEngine worker started");

            loop {
                if let Ok(command) = receiver.recv_timeout(std::time::Duration::from_millis(100)) {
                    match command {
                        StorageCommand::SaveDocument(command) => {
                            let mut document = command.document;
                            info!(target: LOG_TARGET, "Saving document: {:?}", document);

                            if let Err(e) = document.save(&mut conn) {
                                error!(target: LOG_TARGET, "Failed to save document: {:?}", e);
                            }

                            if let Some(resp_tx) = command.resp_tx {
                                let _ = resp_tx.send(Ok(()));
                            } else {
                                warn!(target: LOG_TARGET, "No response channel provided for SaveDocument command");
                            }
                        }
                        StorageCommand::SaveBulkDocuments(command) => {
                            let documents = command.documents;
                            info!(target: LOG_TARGET, "Saving bulk documents: {:?}", documents);

                            if let Err(e) = Document::save_bulk(&mut conn, documents) {
                                error!(target: LOG_TARGET, "Failed to save bulk documents: {:?}", e);
                            }

                            if let Some(resp_tx) = command.resp_tx {
                                let _ = resp_tx.send(Ok(()));
                            } else {
                                warn!(target: LOG_TARGET, "No response channel provided for SaveBulkDocuments command");
                            }
                        }
                        _ => {
                            error!(target: LOG_TARGET, "Unknown storage command received");
                        }
                    }
                }
            }
        });
        
        self.thread_handle = Some(handle);
    }
}
