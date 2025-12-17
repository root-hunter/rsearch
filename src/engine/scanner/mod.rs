pub mod filters;

use std::path::Path;

use tracing::{error, info};

use crate::{engine::{PipelineStage, Sender, extractor::Extractor, scanner::filters::{Filter, FilterError}}, entities::document::{Document, DocumentStatus}};

const LOG_TARGET: &str = "scanner";

#[derive(Debug)]
pub enum ScannerError {
    IoError(std::io::Error),
    FilterError(FilterError),
}

#[derive(Debug, Clone)]
pub enum FiltersMode {
    And,
    Or,
}

#[derive(Debug)]
pub struct Scanner {
    extractor: Extractor,
    filters: Vec<Filter>,
    filters_mode: FiltersMode,
    last_channel_index: usize,
}

impl Scanner {
    pub fn new(extractor: Extractor) -> Self {
        Scanner {
            extractor,
            filters_mode: FiltersMode::And,
            filters: Vec::new(),
            last_channel_index: 0,
        }
    }

    pub fn check_filters(&self, path: &Path) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        for filter in &self.filters {
            match self.filters_mode {
                FiltersMode::And => {
                    if !filter.check(path) {
                        return false;
                    }
                }
                FiltersMode::Or => {
                    if filter.check(path) {
                        return true;
                    }
                }
            }
        }

        match self.filters_mode {
            FiltersMode::And => true,
            FiltersMode::Or => false,
        }
    }

    pub fn set_filters_mode(&mut self, mode: FiltersMode) {
        self.filters_mode = mode;
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    fn process_document(&mut self, document: Document) {
        let channel = self.extractor.get_sender_at(self.last_channel_index);
        
        channel.as_ref().map(|tx| {
            if let Err(e) = tx.send(document.clone()) {
                error!(target: LOG_TARGET, "Failed to send document to extractor: {:?}", e);
            }
        });

        self.last_channel_index = (self.last_channel_index + 1) % self.extractor.get_workers_len();
    }

    pub fn scan_folder(&mut self, path: &str) {
        info!(target: LOG_TARGET, "Scanning folder: {}", path);

        let walker = walkdir::WalkDir::new(path);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();

            if self.check_filters(file_path) {
                info!(target: LOG_TARGET, "Found file: {:?}", file_path);
                let mut document = Document::from_path(file_path);
                document.set_status(DocumentStatus::Scanned);

                self.process_document(document);
            }
        }
    }
}
