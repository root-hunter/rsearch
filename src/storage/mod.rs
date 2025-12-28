pub mod commands;
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/storage_constants.rs"));
}

use std::{collections::HashMap, env, path::MAIN_SEPARATOR, thread::JoinHandle};

use once_cell::sync::Lazy;
use rusqlite::Result;
use tracing::{error, info, warn};

use crate::{
    engine::{EngineError, EngineTask, Receiver, Sender, unbounded_channel},
    entities::{
        container::{self, Container},
        document::Document,
    },
    storage::commands::StorageCommand,
};

const LOG_TARGET: &str = "storage";

const STORAGE_WORKER_RECEIVE_TIMEOUT_MS: Lazy<u64> = Lazy::new(|| {
    env::var("STORAGE_WORKER_RECEIVE_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(constants::DEFAULT_STORAGE_WORKER_RECEIVE_TIMEOUT_MS)
});

const STORAGE_DB_JOURNAL_MODE: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("STORAGE_DB_JOURNAL_MODE")
            .unwrap_or_else(|_| constants::DEFAULT_STORAGE_DB_JOURNAL_MODE.into())
            .into_boxed_str(),
    )
});
const STORAGE_DB_CACHE_SIZE: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("STORAGE_DB_CACHE_SIZE")
            .unwrap_or_else(|_| constants::DEFAULT_STORAGE_DB_CACHE_SIZE.into())
            .into_boxed_str(),
    )
});
const STORAGE_DB_TEMP_STORE: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("STORAGE_DB_TEMP_STORE")
            .unwrap_or_else(|_| constants::DEFAULT_STORAGE_DB_TEMP_STORE.into())
            .into_boxed_str(),
    )
});
const STORAGE_DB_LOCKING_MODE: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("STORAGE_DB_LOCKING_MODE")
            .unwrap_or_else(|_| constants::DEFAULT_STORAGE_DB_LOCKING_MODE.into())
            .into_boxed_str(),
    )
});

pub static STORAGE_DATABASE_PATH: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("STORAGE_DATABASE_PATH")
            .unwrap_or_else(|_| constants::DEFAULT_STORAGE_DB_PATH.into())
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

pub type StorageChannelTx = Sender<StorageCommand>;
pub type StorageChannelRx = Receiver<StorageCommand>;

#[derive(Debug)]
pub struct StorageEngine {
    channel_tx: StorageChannelTx,
    channel_rx: StorageChannelRx,
}

impl Default for StorageEngine {
    fn default() -> Self {
        let (tx, rx) = unbounded_channel::<StorageCommand>();

        Self {
            channel_tx: tx,
            channel_rx: rx,
        }
    }
}

impl StorageEngine {
    pub fn initialize() -> Result<(), StorageError> {
        let conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH)
            .map_err(StorageError::InitializationError)?;

        info!(target: LOG_TARGET, "Initializing storage engine");

        info!(target: LOG_TARGET, "Setting SQLite pragmas");

        conn.pragma_update(None, "journal_mode", *STORAGE_DB_JOURNAL_MODE)
            .map_err(StorageError::InitializationError)?;

        conn.pragma_update(None, "cache_size", *STORAGE_DB_CACHE_SIZE)
            .map_err(StorageError::InitializationError)?;
        conn.pragma_update(None, "temp_store", *STORAGE_DB_TEMP_STORE)
            .map_err(StorageError::InitializationError)?;
        conn.pragma_update(None, "locking_mode", *STORAGE_DB_LOCKING_MODE)
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

impl EngineTask<StorageChannelTx, StorageChannelRx> for StorageEngine {
    fn get_channel_tx(&self) -> &StorageChannelTx {
        &self.channel_tx
    }

    fn get_channel_rx(&self) -> &StorageChannelRx {
        &self.channel_rx
    }

    fn run(&mut self) -> Result<JoinHandle<()>, EngineError> {
        let conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH)
            .map_err(StorageError::InitializationError);

        if let Err(e) = conn {
            error!(target: LOG_TARGET, "Failed to open database connection: {:?}", e);
            return Err(EngineError::StorageError(e));
        }

        let mut conn = conn.unwrap();

        let receiver = self.channel_rx.clone();

        let handle = std::thread::spawn(move || {
            info!(target: LOG_TARGET, "StorageEngine worker started");

            let mut container_cache: HashMap<String, Container> = HashMap::new();

            loop {
                if let Ok(command) = receiver
                    .recv_timeout(std::time::Duration::from_millis(*STORAGE_WORKER_RECEIVE_TIMEOUT_MS))
                {
                    match command {
                        StorageCommand::SaveDocument {
                            mut document,
                            resp_tx,
                        } => {
                            info!(target: LOG_TARGET, "Saving document: {:?}", document);

                            if let Err(e) = document.save(&conn) {
                                error!(target: LOG_TARGET, "Failed to save document: {:?}", e);
                            }

                            if let Some(resp_tx) = resp_tx {
                                let _ = resp_tx.send(Ok(()));
                            } else {
                                warn!(target: LOG_TARGET, "No response channel provided for SaveDocument command");
                            }
                        }
                        StorageCommand::SaveBulkDocuments { documents, resp_tx } => {
                            info!(target: LOG_TARGET, "Saving bulk documents");

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
                    }
                }
            }
        });

        Ok(handle)
    }

    fn name(&self) -> &str {
        LOG_TARGET
    }
}
