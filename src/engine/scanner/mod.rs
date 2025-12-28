pub mod filters;

use std::{
    path::Path,
    thread::{self, JoinHandle},
};

use tracing::{error, info};

use crate::{
    engine::{
        Receiver, Sender,
        extractor::{ExtractorChannelTx, commands::ExtractorCommand},
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

pub type ScannerChannelTx = Sender<String>;
pub type ScannerChannelRx = Receiver<String>;

#[derive(Debug, Clone)]
pub struct Scanner {
    filters: Vec<Filter>,
    filters_mode: FiltersMode,
    channel_tx: ScannerChannelTx,
    channel_rx: ScannerChannelRx,
    channel_extractor_tx: ExtractorChannelTx,
}

impl Scanner {
    pub fn new(
        channel_tx: ScannerChannelTx,
        channel_rx: ScannerChannelRx,
        channel_extractor_tx: ExtractorChannelTx,
    ) -> Self {
        Scanner {
            filters: Vec::new(),
            filters_mode: FiltersMode::And,
            channel_tx,
            channel_rx,
            channel_extractor_tx,
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
        let document = ExtractorCommand::ProcessDocument(ScannedDocument {
            container_type: ContainerType::Folder, // You might want to set this appropriately
            document: document.clone(),
        });

        if let Err(e) = self.channel_extractor_tx.send(document) {
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

    pub fn init(&mut self) -> Result<Vec<JoinHandle<()>>, ScannerError> {
        info!(target: LOG_TARGET, "Scanner is running");

        let mut handles = Vec::new();
        let mut scanner = self.clone();

        let handle = thread::spawn(move || {
            while let Ok(path) = scanner.channel_rx.recv() {
                scanner.scan_folder(&path);
            }
        });

        handles.push(handle);

        Ok(handles)
    }
}
