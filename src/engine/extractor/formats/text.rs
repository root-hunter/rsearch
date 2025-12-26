use crate::engine::extractor::formats::FileExtractor;

pub struct TextExtractor;

impl FileExtractor for TextExtractor {
    fn extract_text(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }
}