pub mod formats;
pub mod worker;
pub mod utils;

use std::{
    env,
    time::Duration,
};

use crate::{
    engine::{EngineTask, PipelineStage, Sender, extractor::worker::ExtractorWorker},
    entities::document::Document, storage::commands::StorageCommand,
};
use once_cell::sync::Lazy;
use tracing::info;

const LOG_TARGET: &str = "extractor";

static EXTRACTOR_INSERT_BATCH_SIZE: Lazy<usize> = Lazy::new(|| {
    env::var("EXTRACTOR_INSERT_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
});

static EXTRACTOR_FLUSH_INTERVAL: Lazy<Duration> = Lazy::new(|| {
    env::var("EXTRACTOR_FLUSH_INTERVAL")
        .ok()
        .and_then(|s| s.parse::<u64>().ok()) // prova a parsare u64
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(5000))
});

#[derive(Debug)]
pub enum ExtractorError {
    ExtractionFailed,
    JoinHandleError,
    IoError(std::io::Error),
}

#[derive(Debug)]
pub struct Extractor {
    database_tx: Sender<StorageCommand>,
    workers: Vec<ExtractorWorker>,
}

impl Extractor {
    pub fn new(database_tx: Sender<StorageCommand>) -> Self {
        Extractor {
            database_tx,
            workers: Vec::new(),
        }
    }
}
    
impl PipelineStage<Document> for Extractor {
    fn get_channel_senders(&self) -> Vec<Sender<Document>> {
        self.workers
            .iter()
            .map(|worker| worker.get_channel_sender().clone())
            .collect()
    }

    fn get_channel_sender_at(&self, index: usize) -> Option<Sender<Document>> {
        self.workers
            .get(index)
            .map(|worker| worker.get_channel_sender().clone())
    }

    fn add_worker(&mut self) {
        let index = self.workers.len();
        
        info!(target: LOG_TARGET, "Starting extractor worker {}", index);

        let database_tx = self.database_tx.clone();
        
        let mut worker = ExtractorWorker::new(index, database_tx);
        worker.run();
        self.workers.push(worker);
    }
}