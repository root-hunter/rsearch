pub mod filters;

use std::path::Path;

use tracing::info;

use crate::{engine::scanner::filters::{Filter, FilterError}, entities::document::Document};

const LOG_TARGET: &str = "scanner";

#[derive(Debug)]
pub enum ScannerError {
    SaveDocuments,
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
    documents: Vec<Document>,
    channel_tx: Option<crossbeam::channel::Sender<Document>>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            filters_mode: FiltersMode::And,
            documents: Vec::new(),
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

    fn add_document(&mut self, document: Document) {
        self.channel_tx.as_ref().map(|tx| tx.send(document.clone()).unwrap());
        //self.documents.push(document);
    }

    pub fn scan_folder(&mut self, path: &str) {
        info!(target: LOG_TARGET, "Scanning folder: {}", path);

        let walker = walkdir::WalkDir::new(path);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();

            if self.check_filters(file_path) {
                info!(target: LOG_TARGET, "Found file: {:?}", file_path);

                self.add_document(Document::from_path(file_path));
            }
        }
    }

    pub fn save_documents(&mut self, conn: &mut rusqlite::Connection) -> Result<(), ScannerError> {
        Document::save_bulk(conn, self.documents.clone()).map_err(|_| ScannerError::SaveDocuments)?;

        Ok(())
    }
}
