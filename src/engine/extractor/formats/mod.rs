pub mod pdf;

pub trait FormatExtractor {
    fn extract_text(&self, path: &str) -> Result<String, Box<dyn std::error::Error>>;
}