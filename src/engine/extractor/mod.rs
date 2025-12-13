pub mod formats;

use std::{env, time::{Duration, Instant}};

use crate::{
    engine::extractor::formats::{FormatExtractor, FormatType},
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
         .and_then(|s| s.parse::<u64>().ok())   // prova a parsare u64
        .map(Duration::from_secs)  
        .unwrap_or(Duration::from_secs(5))
});

#[derive(Debug)]
pub enum ExtractorError {
    ExtractionFailed,
    IoError(std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Extractor {
    extractor_type: FormatType,
    channel_tx: crossbeam::channel::Sender<Document>,
    channel_rx: crossbeam::channel::Receiver<Document>,
}

impl Extractor {
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded::<Document>();

        Extractor {
            channel_tx: tx,
            channel_rx: rx,
            extractor_type: FormatType::Unknown,
        }
    }

    pub fn set_extractor_type(&mut self, extractor_type: FormatType) {
        self.extractor_type = extractor_type;
    }

    pub fn get_extractor_type(&self) -> &FormatType {
        &self.extractor_type
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

    pub fn process_documents(
        &mut self,
        conn: &mut rusqlite::Connection,
    ) -> Result<(), ExtractorError> {
        let mut buffer: Vec<Document> = vec![];
        let mut last_flush = Instant::now();

        loop {
            match self.channel_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(mut document) => {
                    info!(target: LOG_TARGET, "Processing document: {:?}", document);

                    match document.get_format_type() {
                        FormatType::Pdf => {
                            let extractor = formats::pdf::PdfExtractor;
                            match extractor.extract_text(document.get_path()) {
                                Ok(text) => {
                                    let content = text.chars().take(100).collect::<String>();

                                    info!(target: LOG_TARGET, "Extracted text from PDF: {}", content);

                                    document.set_content(content);

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
                Self::flush_buffer(conn, &mut buffer)?;
                last_flush = Instant::now();
            }

            if !buffer.is_empty() && last_flush.elapsed() >= *EXTRACTOR_FLUSH_INTERVAL {
                Self::flush_buffer(conn, &mut buffer)?;
                last_flush = Instant::now();
            }
        }

        Ok(())
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
