pub mod filters;

use std::path::Path;

use crate::{engine::{scanner::filters::Filter, storage::StorageEngine}, entities::document::Document};

pub enum ScannerError {
    IoError(std::io::Error),
    FilterError(ScannerFilterError),
}

pub enum ScannerFilterError {
    InvalidRegex(regex::Error),
}

#[derive(Debug)]
pub struct Scanner {
    filters: Vec<Filter>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            filters: Vec::new(),
        }
    }

    pub fn check_filters(&self, path: &Path) -> bool {
        for filter in &self.filters {
            if !filter.check(path) {
                return false;
            }
        }
        true
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    pub fn scan_folder(&self, storage: &StorageEngine, path: &str) {
        // Implementation for scanning a folder
        println!("Scanning folder: {}", path);

        let walker = walkdir::WalkDir::new(path);
        let conn = storage.get_connection();

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();

            if self.check_filters(file_path) {
                println!("Found file: {:?}", file_path);

                let mut document = Document::from_path(file_path);

                document.save(conn).unwrap_or_else(|e| {
                    eprintln!("Error saving document {:?}: {:?}", file_path, e);
                });

                println!("Document saved: {:?}", document);
            }
        }
    }
}
