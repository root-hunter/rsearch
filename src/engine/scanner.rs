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
pub struct ScannerFilterStringCondition {
    substring: String,
    case_sensitive: bool,
}

impl ScannerFilterStringCondition {
    pub fn new(substring: &str, case_sensitive: bool) -> Self {
        ScannerFilterStringCondition {
            substring: substring.to_string(),
            case_sensitive,
        }
    }

    pub fn matches(&self, target: &str) -> bool {
        if self.case_sensitive {
            target.contains(&self.substring)
        } else {
            target
                .to_lowercase()
                .contains(&self.substring.to_lowercase())
        }
    }
}

#[derive(Debug)]
pub struct ScannerFilter {
    case_sensitive: bool,
    filename_contains: Option<ScannerFilterStringCondition>,
    filename_not_contains: Option<ScannerFilterStringCondition>,
    dir_contains: Option<ScannerFilterStringCondition>,
    dir_not_contains: Option<ScannerFilterStringCondition>,
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
        self.filename_contains = Some(ScannerFilterStringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
    }

    pub fn set_filename_not_contains(&mut self, substring: &str) {
        self.filename_not_contains = Some(ScannerFilterStringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
    }

    pub fn set_dir_contains(&mut self, substring: &str) {
        self.dir_contains = Some(ScannerFilterStringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
    }

    pub fn set_dir_not_contains(&mut self, substring: &str) {
        self.dir_not_contains = Some(ScannerFilterStringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
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
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let dir_path = path.parent().and_then(|p| p.to_str()).unwrap_or_default();

        if let Some(ref condition) = self.filename_contains {
            matches = matches && condition.matches(file_name);
        }

        if let Some(ref condition) = self.filename_not_contains {
            matches = matches && !condition.matches(file_name);
        }

        if let Some(ref condition) = self.dir_contains {
            matches = matches && condition.matches(dir_path);
        }

        if let Some(ref condition) = self.dir_not_contains {
            matches = matches && !condition.matches(dir_path);
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
