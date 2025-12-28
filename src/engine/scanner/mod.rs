pub mod filters;

use std::path::Path;

use crossbeam::channel;
use tracing::{error, info};

use crate::{
    engine::{
        Sender,
        scanner::filters::{Filter, FilterError},
    },
    entities::{
        container::ContainerType,
        document::{Document, DocumentStatus},
    },
};

const LOG_TARGET: &str = "scanner";

#[derive(Debug)]
pub enum ScannerError {
    IoError(std::io::Error),
    FilterError(FilterError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FiltersMode {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub struct ScannedDocument {
    pub container_type: ContainerType,
    pub document: Document,
}

#[derive(Debug, Clone)]
pub struct Scanner {
    filters: Vec<Filter>,
    filters_mode: FiltersMode,
    channel_sender: Sender<ScannedDocument>,
}

impl Scanner {
    pub fn new(channel_sender: Sender<ScannedDocument>) -> Self {
        Scanner {
            filters: Vec::new(),
            filters_mode: FiltersMode::And,
            channel_sender,
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
        if let Err(e) = self.channel_sender.send(ScannedDocument {
                container_type: ContainerType::Folder, // You might want to set this appropriately
                document: document.clone(),
            })
        {
            error!(target: LOG_TARGET, "Failed to send document to extractor: {:?}", e);
        }
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
