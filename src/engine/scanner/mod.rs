pub mod filters;

use std::path::Path;

use crate::{engine::{scanner::filters::{Filter, FilterError}, storage::StorageEngine}, entities::document::Document};

#[derive(Debug)]
pub enum ScannerError {
    SaveDocuments,
    IoError(std::io::Error),
    FilterError(FilterError),
}

#[derive(Debug)]
pub enum ScannerFilterMode {
    And,
    Or,
}

#[derive(Debug)]
pub struct Scanner {
    filter_mode: ScannerFilterMode,
    documents: Vec<Document>,
    filters: Vec<Filter>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            filter_mode: ScannerFilterMode::And,
            documents: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn check_filters(&self, path: &Path) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        for filter in &self.filters {
            match self.filter_mode {
                ScannerFilterMode::And => {
                    if !filter.check(path) {
                        return false;
                    }
                }
                ScannerFilterMode::Or => {
                    if filter.check(path) {
                        return true;
                    }
                }
            }
        }

        match self.filter_mode {
            ScannerFilterMode::And => true,
            ScannerFilterMode::Or => false,
        }
    }

    pub fn set_filter_mode(&mut self, mode: ScannerFilterMode) {
        self.filter_mode = mode;
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    fn add_document(&mut self, document: Document) {
        self.documents.push(document);
    }

    pub fn scan_folder(&mut self, path: &str) {
        println!("Scanning folder: {}", path);

        let walker = walkdir::WalkDir::new(path);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();

            if self.check_filters(file_path) {
                println!("Found file: {:?}", file_path);
                self.add_document(Document::from_path(file_path));
            }
        }
    }

    pub fn save_documents(&mut self, storage: &mut StorageEngine) -> Result<(), ScannerError> {
        let conn = storage.get_connection_mut();

        Document::save_bulk(conn, self.documents.clone()).map_err(|_| ScannerError::SaveDocuments)?;

        Ok(())
    }
}
