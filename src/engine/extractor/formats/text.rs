use crate::engine::extractor::formats::{DataExtracted, FileExtractor};

const LOG_TARGET: &str = "extractor_text";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextExtractor;

impl FileExtractor for TextExtractor {
    fn extract(&self, path: &str) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(DataExtracted::Text(content))    
    }
}