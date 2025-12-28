pub mod formats;
pub mod tokens;
pub mod worker;

use std::{env, time::Duration};

use crate::{
    engine::{
        EngineTask, PipelineStage, Receiver, Sender,
        extractor::worker::ExtractorWorker,
        scanner::{ScannedDocument, Scanner},
    },
    storage::commands::StorageCommand,
};
use crossbeam::channel;
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
    scanner: Scanner,
    database_tx: Sender<StorageCommand>,
    workers: Vec<ExtractorWorker>,
    channel_tx: Sender<ScannedDocument>,
    channel_rx: Receiver<ScannedDocument>,
}

impl Extractor {
    pub fn new(
        database_tx: Sender<StorageCommand>,
        scanner: Scanner,
        channel_tx: Sender<ScannedDocument>,
        channel_rx: Receiver<ScannedDocument>,
    ) -> Self {
        Extractor {
            scanner,
            database_tx,
            workers: Vec::new(),
            channel_tx,
            channel_rx,
        }
    }
}

impl PipelineStage<ScannedDocument> for Extractor {
    fn add_worker(&mut self) {
        let index = self.workers.len();

        info!(target: LOG_TARGET, "Starting extractor worker {}", index);

        let database_tx = self.database_tx.clone();
        let channel_sender = self.channel_tx.clone();
        let channel_receiver = self.channel_rx.clone();

        let mut worker = ExtractorWorker::new(
            index,
            database_tx,
            self.scanner.clone(),
            channel_sender,
            channel_receiver,
        );
        worker.run();
        self.workers.push(worker);
    }
}
