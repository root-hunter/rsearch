use std::time::{Duration, Instant};

use crate::{
    engine::{
        extractor::{
            self, EXTRACTOR_FLUSH_INTERVAL, EXTRACTOR_INSERT_BATCH_SIZE, ExtractorError,
            formats::{self, FormatExtractor, FormatType}, utils::build_text_content,
        },
        storage::STORAGE_DATABASE_PATH,
    },
    entities::document::{Document, DocumentStatus},
};
use crossbeam::channel;
use tracing::{error, info};

const LOG_TARGET: &str = "extractor_worker";

#[derive(Debug)]
pub struct ExtractorWorker {
    id: usize,
    channel_tx: crossbeam::channel::Sender<Document>,
    channel_rx: crossbeam::channel::Receiver<Document>,
    pub thread_handle: Option<std::thread::JoinHandle<Result<(), ExtractorError>>>,
}

impl ExtractorWorker {
    pub fn new(id: usize) -> Self {
        let (tx, rx) = channel::unbounded::<Document>();

        ExtractorWorker {
            id,
            channel_tx: tx,
            channel_rx: rx,
            thread_handle: None,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn run(&mut self) {
        assert!(self.thread_handle.is_none(), "Worker is already running");

        let receiver = self.channel_rx.clone();
        let conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH);

        let worker_id = self.id;

        if let Err(e) = conn {
            error!(target: LOG_TARGET, worker = worker_id, "Failed to open database connection: {:?}", e);
            return;
        }

        let mut conn = conn.unwrap();

        self.thread_handle = Some(std::thread::spawn(move || {
            let mut buffer: Vec<Document> = vec![];
            let mut last_flush = Instant::now();

            loop {
                match receiver.recv_timeout(Duration::from_millis(200)) {
                    Ok(mut document) => {
                        info!(target: LOG_TARGET, worker_id = worker_id, "Processing document: {:?}", document);

                        match document.get_format_type() {
                            FormatType::Pdf => {
                                let extractor = formats::pdf::PdfExtractor;
                                match extractor.extract_text(document.get_path()) {
                                    Ok(text) => {
                                        info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text from PDF, length: {}", text.len());

                                        let content = build_text_content(text);

                                        document.set_content(content);
                                        document.set_status(DocumentStatus::Extracted);

                                        buffer.push(document);
                                    }
                                    Err(e) => {
                                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract text from PDF: {:?} ({})", e, document.get_path());
                                    }
                                }
                            }
                            FormatType::Txt => {
                                error!(target: LOG_TARGET, worker_id = worker_id, "Text extraction not implemented yet.");
                            }
                            FormatType::Docx => {
                                let extractor = formats::microsoft::docx::DocxExtractor;
                                match extractor.extract_text(document.get_path()) {
                                    Ok(text) => {
                                        info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text from DOCX, length: {}", text.len());

                                        let content: String = build_text_content(text);
                                        info!(target: LOG_TARGET, "Extracted text distribution: {:?}", content);

                                        document.set_content(content);
                                        document.set_status(DocumentStatus::Extracted);

                                        buffer.push(document);
                                    }
                                    Err(e) => {
                                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract text from DOCX: {:?} ({})", e, document.get_path());
                                    }
                                }
                            }
                            _ => {
                                error!(target: LOG_TARGET, worker_id = worker_id, "Unsupported document format: {:?}", document.get_format_type());
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
