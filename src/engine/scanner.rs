use std::path::Path;

use regex::Regex;

use crate::{engine::storage::StorageEngine, entities::document::Document};

pub enum ScannerError {
    IoError(std::io::Error),
    FilterError(ScannerFilterError),
}

pub enum ScannerFilterError {
    InvalidRegex(regex::Error),
}

#[derive(Debug)]
pub struct ScannerFilter {
    case_sensitive: bool,
    filename_contains: Option<String>,
    filename_not_contains: Option<String>,
    dir_contains: Option<String>,
    dir_not_contains: Option<String>,
    extension_is: Option<String>,
    extension_is_not: Option<String>,
    filename_regex: Option<Regex>,
}

impl ScannerFilter {
    pub fn new() -> Self {
        ScannerFilter {
            case_sensitive: true,
            filename_contains: None,
            filename_not_contains: None,
            dir_contains: None,
            dir_not_contains: None,
            extension_is: None,
            extension_is_not: None,
            filename_regex: None,
        }
    }

    pub fn set_filename_contains(&mut self, substring: &str) {
        self.filename_contains = Some(substring.to_string());
    }

    pub fn set_filename_not_contains(&mut self, substring: &str) {
        self.filename_not_contains = Some(substring.to_string());
    }

    pub fn set_dir_contains(&mut self, substring: &str) {
        self.dir_contains = Some(substring.to_string());
    }

    pub fn set_dir_not_contains(&mut self, substring: &str) {
        self.dir_not_contains = Some(substring.to_string());
    }

    pub fn set_extension_is(&mut self, extension: &str) {
        self.extension_is = Some(extension.to_string());
    }

    pub fn set_extension_is_not(&mut self, extension: &str) {
        self.extension_is_not = Some(extension.to_string());
    }

    pub fn set_filename_regex(&mut self, pattern: &str) -> Result<(), ScannerFilterError> {
        let regex = Regex::new(pattern).map_err(ScannerFilterError::InvalidRegex)?;
        self.filename_regex = Some(regex);
        Ok(())
    }

    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }

    pub fn check(&self, path: &Path) -> bool {
        let mut matches = true;
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        let dir_path = path.parent().and_then(|p| p.to_str()).unwrap_or_default();

        if let Some(ref substring) = self.filename_contains {
            if self.case_sensitive {
                matches = matches && file_name.contains(substring);
            } else {
                matches = matches && file_name.to_lowercase().contains(&substring.to_lowercase());
            }
        }

        if let Some(ref substring) = self.filename_not_contains {
            if self.case_sensitive {
                matches = matches && !file_name.contains(substring);
            } else {
                matches = matches && !file_name.to_lowercase().contains(&substring.to_lowercase());
            }
        }

        if let Some(ref substring) = self.dir_contains {
            if self.case_sensitive {
                matches = matches && dir_path.contains(substring);
            } else {
                matches = matches && dir_path.to_lowercase().contains(&substring.to_lowercase());
            }
        }

        if let Some(ref substring) = self.dir_not_contains {
            if self.case_sensitive {
                matches = matches && !dir_path.contains(substring);
            } else {
                matches = matches && !dir_path.to_lowercase().contains(&substring.to_lowercase());
            }
        }

        if let Some(ref extension) = self.extension_is {
            if let Some(file_extension) = path.extension().and_then(|ext| ext.to_str()) {
                matches = matches && file_extension == extension;
            } else {
                matches = false;
            }
        }

        if let Some(ref extension) = self.extension_is_not {
            if let Some(file_extension) = path.extension().and_then(|ext| ext.to_str()) {
                matches = matches && file_extension != extension;
            } else {
                matches = false;
            }
        }

        if let Some(ref regex) = self.filename_regex {
            matches = matches && regex.is_match(file_name);
        }

        matches
    }
}

#[derive(Debug)]
pub struct Scanner {
    filters: Vec<ScannerFilter>,
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

    pub fn add_filter(&mut self, filter: ScannerFilter) {
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