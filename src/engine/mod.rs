use std::thread::JoinHandle;

use crossbeam::channel;

use crate::storage::StorageError;

pub mod classifier;
pub mod extractor;
pub mod scanner;
pub mod utils;
pub mod decompressor;

//const LOG_TARGET: &str = "engine";

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

#[derive(Debug, Default)]
pub struct Engine {
    pub classifier: classifier::Classifier,
}

pub trait PipelineStage<T> {
    fn init(&mut self, num_workers: usize) -> Result<Vec<JoinHandle<()>>, EngineError> {
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            handles.push(self.add_worker()?);
        }
        Ok(handles)
    }

    fn add_worker(&mut self) -> Result<JoinHandle<()>, EngineError>;
}

pub trait EngineTask<T> {
    fn name(&self) -> &str;
    fn run(&mut self) -> Result<JoinHandle<()>, EngineError>;
    fn get_channel_sender(&self) -> &channel::Sender<T>;
    fn get_channel_receiver(&self) -> &channel::Receiver<T>;
}

pub trait EngineTaskWorker<T>: EngineTask<T> {
    fn get_id(&self) -> usize;
}
