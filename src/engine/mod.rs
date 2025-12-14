use crossbeam::channel;
use tracing::info;

use crate::storage::{StorageEngine, StorageError};

pub mod classifier;
pub mod extractor;
pub mod scanner;
pub mod utils;

const LOG_TARGET: &str = "engine";

#[derive(Debug)]
pub enum EngineError {
    StorageError(StorageError),
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

        StorageEngine::initialize(conn).map_err(EngineError::StorageError)
    }
}

pub trait EngineTask<T> {
    fn new(id: usize) -> Self
    where
        Self: Sized;
    fn run(&mut self);
    fn get_channel_sender(&self) -> &channel::Sender<T>;
    fn get_channel_receiver(&self) -> &channel::Receiver<T>;
}

pub trait EngineTaskWorker {
    fn get_id(&self) -> usize;
}
