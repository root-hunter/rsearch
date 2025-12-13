pub mod formats;

use std::{
    env,
    time::{Duration, Instant},
};

use crate::{
    engine::{
        extractor::formats::{FormatExtractor, FormatType},
        storage::STORAGE_DATABASE_PATH,
    },
    entities::document::Document,
};
use crossbeam::channel;
use once_cell::sync::Lazy;
use tracing::{error, info};

const LOG_TARGET: &str = "extractor";

static EXTRACTOR_INSERT_BATCH_SIZE: Lazy<usize> = Lazy::new(|| {
    env::var("BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
});

static EXTRACTOR_FLUSH_INTERVAL: Lazy<Duration> = Lazy::new(|| {
    env::var("FLUSH_INTERVAL")
        .ok()
        .and_then(|s| s.parse::<u64>().ok()) // prova a parsare u64
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(5))
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
            let mut worker = ExtractorWorker::new();
            worker.run();
            self.workers.push(worker);
        }
    }

    pub fn get_channel_sender_all(&self) -> Vec<channel::Sender<Document>> {
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

#[derive(Debug)]
pub struct ExtractorWorker {
    channel_tx: crossbeam::channel::Sender<Document>,
    channel_rx: crossbeam::channel::Receiver<Document>,
    thread_handle: Option<std::thread::JoinHandle<Result<(), ExtractorError>>>,
}

impl ExtractorWorker {
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded::<Document>();

        ExtractorWorker {
            channel_tx: tx,
            channel_rx: rx,
            thread_handle: None,
        }
    }

    pub fn run(&mut self) {
        let receiver = self.channel_rx.clone();
        let mut conn =
            rusqlite::Connection::open(*STORAGE_DATABASE_PATH).expect("Failed to open database");

        self.thread_handle = Some(std::thread::spawn(move || {
            let mut buffer: Vec<Document> = vec![];
            let mut last_flush = Instant::now();

            loop {
                match receiver.recv_timeout(Duration::from_millis(200)) {
                    Ok(mut document) => {
                        info!(target: LOG_TARGET, "Processing document: {:?}", document);

                        match document.get_format_type() {
                            FormatType::Pdf => {
                                let extractor = formats::pdf::PdfExtractor;
                                match extractor.extract_text(document.get_path()) {
                                    Ok(text) => {
                                        //let content = text.chars().take(100).collect::<String>();
                                        //info!(target: LOG_TARGET, "Extracted text from PDF: {}", content);

                                        document.set_content(text);

                                        buffer.push(document);
                                    }
                                    Err(e) => {
                                        error!(target: LOG_TARGET, "Failed to extract text from PDF: {:?}", e);
                                    }
                                }
                            }
                            FormatType::Txt => {
                                error!(target: LOG_TARGET, "Text extraction not implemented yet.");
                            }
                            _ => {
                                error!(target: LOG_TARGET, "Unsupported document format: {:?}", document.get_format_type());
                            }
                        }
                    }
                    Err(channel::RecvTimeoutError::Timeout) => {
                        // Timeout occurred, check if we need to flush
                    }
                    Err(e) => {
                        error!(target: LOG_TARGET, "Channel receive error: {:?}", e);
                        break;
                    }
                }

                if buffer.len() >= *EXTRACTOR_INSERT_BATCH_SIZE {
                    Self::flush_buffer(&mut conn, &mut buffer)?;
                    last_flush = Instant::now();
                }

                if !buffer.is_empty() && last_flush.elapsed() >= *EXTRACTOR_FLUSH_INTERVAL {
                    Self::flush_buffer(&mut conn, &mut buffer)?;
                    last_flush = Instant::now();
                }
            }
            Ok(())
            // Worker loop would go here
        }));
    }

    pub fn get_channel_sender(&self) -> &channel::Sender<Document> {
        &self.channel_tx
    }

    pub fn get_channel_receiver(&self) -> &channel::Receiver<Document> {
        &self.channel_rx
    }

    pub fn extract(&self, data: &str) {
        // Extraction logic would go here
        info!(target: LOG_TARGET, "Extracting data: {}", data);
    }

    pub fn flush_buffer(
        conn: &mut rusqlite::Connection,
        buffer: &mut Vec<Document>,
    ) -> Result<(), ExtractorError> {
        info!(
            target: LOG_TARGET,
            count = buffer.len(),
            "Saving batch"
        );

        let batch = buffer.drain(..).collect::<Vec<_>>();

        Document::save_bulk(conn, batch).map_err(|_| ExtractorError::ExtractionFailed)?;

        Ok(())
    }
}
