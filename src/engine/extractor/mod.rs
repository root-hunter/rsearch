pub mod formats;
pub mod worker;

use std::{
    env,
    time::Duration,
};

use crate::{
    engine::extractor::worker::ExtractorWorker,
    entities::document::Document,
};
use crossbeam::channel;
use once_cell::sync::Lazy;
use tracing::{error, info};

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
    workers: Vec<ExtractorWorker>,
}

impl Extractor {
    pub fn new() -> Self {
        Extractor {
            workers: Vec::new(),
        }
    }

    pub fn init(&mut self, num_workers: usize) {
        for _ in 0..num_workers {
            let index = self.workers.len();
            
            info!(target: LOG_TARGET, "Starting extractor worker {}", index);
            
            let mut worker = ExtractorWorker::new(index);
            worker.run();
            self.workers.push(worker);
        }
    }

    pub fn get_channel_senders(&self) -> Vec<channel::Sender<Document>> {
        self.workers
            .iter()
            .map(|worker| worker.get_channel_sender().clone())
            .collect()
    }

    pub fn get_channel_sender_at(&self, index: usize) -> Option<channel::Sender<Document>> {
        self.workers
            .get(index)
            .map(|worker| worker.get_channel_sender().clone())
    }

    pub fn join_all(&mut self) -> Result<(), ExtractorError> {
        loop {
            if self.workers.iter().all(|w| w.thread_handle.is_none()) {
                break;
            }

            for worker in &mut self.workers {
                if let Some(handle) = worker.thread_handle.take() {
                    if let Err(e) = handle.join() {
                        error!(target: LOG_TARGET, "Extractor worker error: {:?}", e);
                    } else {
                        info!(target: LOG_TARGET, "Extractor worker joined successfully");
                        worker.run();
                    }
                }
            }
        }

        Ok(())
    }
}