pub mod filters;

use std::path::Path;

use crate::{engine::{scanner::filters::{Filter, FilterError}, storage::StorageEngine}, entities::document::Document};

pub enum ScannerError {
    IoError(std::io::Error),
    FilterError(FilterError),
}

#[derive(Debug)]
pub struct Scanner {
    documents: Vec<Document>,
    filters: Vec<Filter>,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            documents: Vec::new(),
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

    pub fn save_documents(&mut self, storage: &StorageEngine) {
        let conn = storage.get_connection();

        for document in &mut self.documents {
            document.save(conn).unwrap_or_else(|e| {
                eprintln!("Error saving document {:?}: {:?}", document.get_path(), e);
            });
            println!("Document saved: {:?}", document);
        }

        println!("Saved {} documents.", self.documents.len());
    }
}
