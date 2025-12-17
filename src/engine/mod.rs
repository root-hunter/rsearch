use crossbeam::channel;

use crate::{entities::document::Document, storage::StorageError};

pub mod classifier;
pub mod extractor;
pub mod scanner;
pub mod utils;

const LOG_TARGET: &str = "engine";

pub type Sender<T> = crossbeam::channel::Sender<T>;
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
pub fn unbounded_channel<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::unbounded::<T>()
}
pub type ChannelRecvTimeoutError = crossbeam::channel::RecvTimeoutError;

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
