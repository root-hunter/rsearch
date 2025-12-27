pub mod commands;

use std::{collections::HashMap, env, path::MAIN_SEPARATOR};

use once_cell::sync::Lazy;
use rusqlite::Result;
use tracing::{error, info, warn};

use crate::{
    engine::{EngineTask, Receiver, Sender, unbounded_channel},
    entities::{
        container::{self, Container},
        document::Document,
    },
    storage::commands::StorageCommand,
};

const LOG_TARGET: &str = "storage";
const WORKER_RECEIVE_TIMEOUT_MS: u64 = 100;

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
    ContainerError(container::ContainerError),
    DocumentError(crate::entities::document::DocumentError),
}

#[derive(Debug)]
pub struct StorageEngine {
    channel_tx: Sender<StorageCommand>,
    channel_rx: Receiver<StorageCommand>,
    thread_handle: Option<std::thread::JoinHandle<Result<(), StorageError>>>,
}

impl StorageEngine {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel::<StorageCommand>();

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
            "CREATE TABLE IF NOT EXISTS containers (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                type TEXT NOT NULL
            )",
            [],
        )
        .map_err(StorageError::InitializationError)?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_containers_path ON containers(path)",
            [],
        )
        .map_err(StorageError::InitializationError)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL,
                extension TEXT,
                status TEXT NOT NULL DEFAULT 'New',
                container_id INTEGER NOT NULL,
                UNIQUE(filename, container_id),
                FOREIGN KEY(container_id) REFERENCES containers(id)
            )",
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

        info!(target: LOG_TARGET, "Create view for full document info");

        conn.execute(
            &format!(
                "CREATE VIEW IF NOT EXISTS documents_view AS 
                    SELECT 
                    c.id as container_id,
                    d.id as id,
                    c.type as container_type,
                    d.status as status,
                    c.path || '{}' || d.filename as path,
                    c.path as container_path,
                    d.filename as filename,
                    d.extension as extension
                    FROM documents d
                    INNER JOIN containers c ON c.id = d.container_id
                    ORDER BY container_id, id",
                MAIN_SEPARATOR
            ),
            [],
        )
        .map_err(StorageError::InitializationError)?;

        info!(target: LOG_TARGET, "Storage engine initialized successfully");

        Ok(())
    }
}

impl EngineTask<StorageCommand> for StorageEngine {
    fn get_channel_sender(&self) -> &Sender<StorageCommand> {
        &self.channel_tx
    }

    fn get_channel_receiver(&self) -> &Receiver<StorageCommand> {
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

            let mut container_cache: HashMap<String, Container> = HashMap::new();

            loop {
                if let Ok(command) = receiver
                    .recv_timeout(std::time::Duration::from_millis(WORKER_RECEIVE_TIMEOUT_MS))
                {
                    match command {
                        StorageCommand::SaveDocument {
                            mut document,
                            resp_tx,
                        } => {
                            info!(target: LOG_TARGET, "Saving document: {:?}", document);

                            if let Err(e) = document.save(&mut conn) {
                                error!(target: LOG_TARGET, "Failed to save document: {:?}", e);
                            }

                            if let Some(resp_tx) = resp_tx {
                                let _ = resp_tx.send(Ok(()));
                            } else {
                                warn!(target: LOG_TARGET, "No response channel provided for SaveDocument command");
                            }
                        }
                        StorageCommand::SaveBulkDocuments { documents, resp_tx } => {
                            info!(target: LOG_TARGET, "Saving bulk documents: {:?}", documents);

                            if let Err(e) = container::Container::update_cache_from_documents(
                                &mut conn,
                                &documents,
                                &mut container_cache,
                            ) {
                                error!(target: LOG_TARGET, "Failed to update container cache from documents: {:?}", e);
                            }

                            if let Err(e) =
                                Document::save_bulk(&mut conn, documents, &mut container_cache)
                            {
                                error!(target: LOG_TARGET, "Failed to save bulk documents: {:?}", e);
                            }

                            if let Some(resp_tx) = resp_tx {
                                let _ = resp_tx.send(Ok(()));
                            } else {
                                warn!(target: LOG_TARGET, "No response channel provided for SaveBulkDocuments command");
                            }
                        }
                        StorageCommand::SaveArchive {
                            mut archive,
                            resp_tx,
                        } => {
                            info!(target: LOG_TARGET, "Saving archive: {:?}", archive);

                            if let Err(e) = archive.save(&mut conn) {
                                error!(target: LOG_TARGET, "Failed to save archive: {:?}", e);

                                if let Some(resp_tx) = resp_tx {
                                    let _ = resp_tx.send(Err(StorageError::ContainerError(e)));
                                }
                                continue;
                            }

                            if let Some(resp_tx) = resp_tx {
                                let _ = resp_tx.send(Ok(archive));
                            } else {
                                warn!(target: LOG_TARGET, "No response channel provided for SaveArchive command");
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

    fn name(&self) -> &str {
        LOG_TARGET
    }
}
