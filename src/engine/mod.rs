use crossbeam::channel;

use crate::storage::StorageError;

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

pub trait Stage<T> {
    fn get_channel_senders(&self) -> Vec<Sender<T>> {
        todo!("Implement get_channel_senders")
    }

    fn get_channel_sender_at(&self, index: usize) -> Option<Sender<T>> {
        todo!("Implement get_channel_sender_at")
    }
}

pub trait PipelineStage<T> {
    fn init(&mut self, num_workers: usize) {
        for _ in 0..num_workers {
            self.add_worker();
        }
    }

    fn get_senders(&self) -> Vec<Sender<T>>;

    fn get_sender_at(&self, index: usize) -> Option<Sender<T>>;

    fn get_workers_len(&self) -> usize;

    fn add_worker(&mut self);
}

pub trait EngineTask<T> {
    fn name(&self) -> &str;
    fn run(&mut self);
    fn get_channel_sender(&self) -> &channel::Sender<T>;
    fn get_channel_receiver(&self) -> &channel::Receiver<T>;
}

pub trait EngineTaskWorker<T>: EngineTask<T> {
    fn get_id(&self) -> usize;
}
