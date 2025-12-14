
use tracing::info;

use crate::engine::storage::StorageEngine;

pub mod storage;
pub mod scanner;
pub mod extractor;
pub mod classifier;
pub mod utils;

const LOG_TARGET: &str = "engine";

#[derive(Debug)]
pub enum EngineError {
    StorageError(storage::StorageError),
    ExtractorError(extractor::ExtractorError),
}

#[derive(Debug)]
pub struct Engine {
    pub classifier: classifier::Classifier,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            classifier: classifier::Classifier::new(),
        }
    }

    pub fn initialize(&mut self, conn: &rusqlite::Connection) -> Result<(), EngineError> {
        info!(target: LOG_TARGET, "Engine starting");

        StorageEngine::initialize(conn)
            .map_err(EngineError::StorageError)
    }
}