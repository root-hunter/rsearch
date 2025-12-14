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
}

pub trait EngineTask<T> {
    fn run(&mut self);
    fn get_channel_sender(&self) -> &channel::Sender<T>;
    fn get_channel_receiver(&self) -> &channel::Receiver<T>;
}

pub trait EngineTaskWorker {
    fn get_id(&self) -> usize;
}
