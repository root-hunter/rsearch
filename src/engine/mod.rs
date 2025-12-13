use tracing::info;

pub mod storage;
pub mod scanner;
pub mod extractor;
pub mod classifier;
pub mod utils;

const LOG_TARGET: &str = "engine";

#[derive(Debug)]
pub enum EngineError {
    StorageError(storage::StorageError),
}

#[derive(Debug)]
pub struct Engine {
    pub storage_engine: storage::StorageEngine,
    pub scanner: scanner::Scanner,
    pub extractor: extractor::Extractor,
    pub classifier: classifier::Classifier,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            storage_engine: storage::StorageEngine::new(),
            scanner: scanner::Scanner::new(),
            extractor: extractor::Extractor::new(),
            classifier: classifier::Classifier::new(),
        }
    }

    pub fn initialize(&mut self) -> Result<(), EngineError> {
        info!(target: LOG_TARGET, "Engine starting");

        self.storage_engine
            .initialize()
            .map_err(EngineError::StorageError)
    }
}