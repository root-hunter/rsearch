use std::thread::JoinHandle;

use crate::storage::StorageError;

pub mod classifier;
pub mod decompressor;
pub mod extractor;
pub mod scanner;
pub mod utils;

//const LOG_TARGET: &str = "engine";

pub type Sender<T> = crossbeam::channel::Sender<T>;
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
pub fn unbounded_channel<T>() -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::unbounded::<T>()
}
pub type ChannelRecvTimeoutError = crossbeam::channel::RecvTimeoutError;

#[derive(Debug)]
pub enum EngineError {
    IoError(std::io::Error),
    ZipError(zip::result::ZipError),
    RusqliteError(rusqlite::Error),
    StorageError(StorageError),
    ExtractorError(extractor::ExtractorError),
}

#[derive(Debug, Default)]
pub struct Engine {
    pub classifier: classifier::Classifier,
}

pub trait PipelineStage {
    fn init(&mut self, num_workers: usize) -> Result<Vec<JoinHandle<()>>, EngineError> {
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            handles.push(self.add_worker()?);
        }
        Ok(handles)
    }

    fn add_worker(&mut self) -> Result<JoinHandle<()>, EngineError>;
}

pub trait EngineTask<S, R> {
    fn name(&self) -> &str;
    fn run(&mut self) -> Result<JoinHandle<()>, EngineError>;
    fn get_channel_tx(&self) -> &S;
    fn get_channel_rx(&self) -> &R;
}

pub trait EngineTaskWorker<S, R>: EngineTask<S, R> {
    fn get_id(&self) -> usize;
}
