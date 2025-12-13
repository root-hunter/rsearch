pub mod formats;

use std::fs;

use crate::entities::document::Document;
use crossbeam::channel;
use tracing::{error, info};

const LOG_TARGET: &str = "extractor";

#[derive(Debug, Clone)]
pub enum ExtractorType {
    Pdf,
    Docx,
    Txt,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Extractor {
    extractor_type: ExtractorType,
    channel_tx: crossbeam::channel::Sender<Document>,
    channel_rx: crossbeam::channel::Receiver<Document>,
}

impl Extractor {
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded::<Document>();

        Extractor {
            channel_tx: tx,
            channel_rx: rx,
            extractor_type: ExtractorType::Unknown,
        }
    }

    pub fn set_extractor_type(&mut self, extractor_type: ExtractorType) {
        self.extractor_type = extractor_type;
    }

    pub fn get_extractor_type(&self) -> &ExtractorType {
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

    pub fn process_documents(&mut self) {
        while let Ok(document) = self.channel_rx.try_recv() {
            info!(target: LOG_TARGET, "Processing document: {:?}", document);

            let path = document.get_path();
            let path = std::path::Path::new(path);

            fs::read_to_string(path)
                .map(|content| {
                    info!(target: LOG_TARGET, "Extracted content: {}", content);
                })
                .unwrap_or_else(|err| {
                    error!(target: LOG_TARGET, "Failed to read file {}: {}", path.display(), err);
                });
        }
    }
}
