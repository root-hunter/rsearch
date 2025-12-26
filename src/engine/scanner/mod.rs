pub mod filters;

use std::path::Path;

use tracing::{error, info};

use crate::{engine::{Sender, scanner::filters::{Filter, FilterError}}, entities::{container::ContainerType, document::{Document, DocumentStatus}}};

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

#[derive(Debug, Clone)]
pub struct ScannedDocument {
    pub container_type: ContainerType, 
    pub document: Document,
}

#[derive(Debug, Clone)]
pub struct Scanner {
    filters: Vec<Filter>,
    filters_mode: FiltersMode,
    channels: Vec<Sender<ScannedDocument>>,
    last_channel_index: usize,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            filters_mode: FiltersMode::And,
            filters: Vec::new(),
            channels: Vec::new(),
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

    pub fn add_channel_sender(&mut self, sender: Sender<ScannedDocument>) {
        self.channels.push(sender);
    }

    pub fn add_channel_senders(&mut self, senders: Vec<Sender<ScannedDocument>>) {
        for sender in senders {
            self.add_channel_sender(sender);
        }
    }

    pub fn remove_channel_sender(&mut self, index: usize) {
        if index < self.channels.len() {
            self.channels.remove(index);
        }
    }

    pub fn clear_channel_senders(&mut self) {
        self.channels.clear();
    }

    pub fn set_filters_mode(&mut self, mode: FiltersMode) {
        self.filters_mode = mode;
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    fn process_document(&mut self, document: Document) {
        let channel = self.channels.get(self.last_channel_index);
        
        channel.as_ref().map(|tx| {
            if let Err(e) = tx.send(ScannedDocument {
                container_type: ContainerType::Folder, // You might want to set this appropriately
                document: document.clone(),
            }) {
                error!(target: LOG_TARGET, "Failed to send document to extractor: {:?}", e);
            }
        });

        self.last_channel_index = (self.last_channel_index + 1) % self.channels.len();
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
