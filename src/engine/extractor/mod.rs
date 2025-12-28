pub mod formats;
pub mod tokens;
pub mod workers;
pub mod commands;
pub mod constants {
    include!(concat!(env!("OUT_DIR"), "/extractor_constants.rs"));
}

use std::{env, thread::JoinHandle, time::Duration};

use crate::{
    engine::{
        EngineError, EngineTask, PipelineStage, Receiver, Sender,
        extractor::{commands::ExtractorCommand, workers::ExtractorWorker},
        scanner::{ScannedDocument, Scanner},
    },
    storage::StorageChannelTx,
};
use once_cell::sync::Lazy;

const LOG_TARGET: &str = "extractor";

static EXTRACTOR_INSERT_BATCH_SIZE: Lazy<usize> = Lazy::new(|| {
    env::var("EXTRACTOR_INSERT_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(constants::DEFAULT_INSERT_BATCH_SIZE)
});

static EXTRACTOR_FLUSH_INTERVAL: Lazy<Duration> = Lazy::new(|| {
    env::var("EXTRACTOR_FLUSH_INTERVAL")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(constants::DEFAULT_FLUSH_INTERVAL_MS))
});

#[derive(Debug)]
pub enum ExtractorError {
    ExtractionFailed,
    JoinHandleError,
    IoError(std::io::Error),
}

pub type ExtractorChannelTx = Sender<ExtractorCommand>;
pub type ExtractorChannelRx = Receiver<ExtractorCommand>;

#[derive(Debug)]
pub struct Extractor {
    scanner: Scanner,
    database_tx: StorageChannelTx,
    workers: Vec<ExtractorWorker>,
    channel_tx: ExtractorChannelTx,
    channel_rx: ExtractorChannelRx,
}

impl Extractor {
    pub fn new(
        database_tx: StorageChannelTx,
        scanner: Scanner,
        channel_tx: ExtractorChannelTx,
        channel_rx: ExtractorChannelRx,
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

impl PipelineStage for Extractor {
    fn add_worker(&mut self) -> Result<JoinHandle<()>, EngineError> {
        let index = self.workers.len();

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
        let handle = worker.run()?;
        self.workers.push(worker);
        Ok(handle)
    }
}
