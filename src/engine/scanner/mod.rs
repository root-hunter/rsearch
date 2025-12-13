pub mod filters;

use std::path::Path;

use tracing::{error, info};

use crate::{engine::scanner::filters::{Filter, FilterError}, entities::document::Document};

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

#[derive(Debug, Clone)]
pub struct Scanner {
    filters: Vec<Filter>,
    filters_mode: FiltersMode,
    channel_tx: Option<crossbeam::channel::Sender<Document>>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            filters_mode: FiltersMode::And,
            filters: Vec::new(),
            channel_tx: None,
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

    pub fn set_channel_sender(&mut self, sender: crossbeam::channel::Sender<Document>) {
        self.channel_tx = Some(sender);
    }

    pub fn set_filters_mode(&mut self, mode: FiltersMode) {
        self.filters_mode = mode;
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    fn process_document(&mut self, document: Document) {
        self.channel_tx.as_ref().map(|tx| {
            if let Err(e) = tx.send(document.clone()) {
                error!(target: LOG_TARGET, "Failed to send document to extractor: {:?}", e);
            }
        });
    }

    pub fn scan_folder(&mut self, path: &str) {
        info!(target: LOG_TARGET, "Scanning folder: {}", path);

        let walker = walkdir::WalkDir::new(path);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();

            if self.check_filters(file_path) {
                info!(target: LOG_TARGET, "Found file: {:?}", file_path);

                self.process_document(Document::from_path(file_path));
            }
        }
    }
}
