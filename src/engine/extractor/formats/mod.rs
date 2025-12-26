pub mod text;
pub mod pdf;
pub mod microsoft;

#[derive(Debug, Clone)]
pub enum FormatType {
    Pdf,
    Docx,
    Text,
    Unknown,
}

impl FormatType {
    pub fn get_by_extension(extension: &str) -> FormatType {
        match extension.to_lowercase().as_str() {
            "pdf" => FormatType::Pdf,
            "docx" => FormatType::Docx,
            "txt" => FormatType::Text,
            _ => FormatType::Unknown,
        }
    }
}

pub trait FormatExtractor {
    fn extract_text(&self, path: &str) -> Result<String, Box<dyn std::error::Error>>;
}
