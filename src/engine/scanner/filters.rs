use std::path::Path;

use regex::Regex;

pub enum ScannerError {
    IoError(std::io::Error),
    FilterError(ScannerFilterError),
}

pub enum ScannerFilterError {
    InvalidRegex(regex::Error),
}

#[derive(Debug)]
pub struct StringCondition {
    substring: String,
    case_sensitive: bool,
}

impl StringCondition {
    pub fn new(substring: &str, case_sensitive: bool) -> Self {
        StringCondition {
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
pub struct Filter {
    case_sensitive: bool,
    filename_contains: Option<StringCondition>,
    filename_not_contains: Option<StringCondition>,
    dir_contains: Option<StringCondition>,
    dir_not_contains: Option<StringCondition>,
    extension_is: Option<String>,
    extension_is_not: Option<String>,
    filename_regex: Option<Regex>,
}

impl Filter {
    pub fn new() -> Self {
        Filter {
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
        self.filename_contains = Some(StringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
    }

    pub fn set_filename_not_contains(&mut self, substring: &str) {
        self.filename_not_contains = Some(StringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
    }

    pub fn set_dir_contains(&mut self, substring: &str) {
        self.dir_contains = Some(StringCondition {
            substring: substring.to_string(),
            case_sensitive: self.case_sensitive,
        });
    }

    pub fn set_dir_not_contains(&mut self, substring: &str) {
        self.dir_not_contains = Some(StringCondition {
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