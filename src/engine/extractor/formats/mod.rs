use crate::{
    engine::scanner::ScannedDocument,
    entities::{container::Container, document::Document},
};

pub mod archive;
pub mod microsoft;
pub mod pdf;
pub mod text;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Archive {
    Zip,
    // Tar,
    // Rar,
    // SevenZ,
    // Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FormatType {
    Pdf,
    Docx,
    Text,
    Archive(Archive),
    Unknown,
}

impl FormatType {
    pub fn get_by_extension(extension: &str) -> FormatType {
        match extension.to_lowercase().as_str() {
            "pdf" => FormatType::Pdf,
            "docx" => FormatType::Docx,
            "txt" => FormatType::Text,
            "zip" => FormatType::Archive(Archive::Zip),
            // "tar" => FormatType::Archive(Archive::Tar),
            // "rar" => FormatType::Archive(Archive::Rar),
            // "7z" => FormatType::Archive(Archive::SevenZ),
            _ => FormatType::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataExtracted {
    Text(String),
    ArchiveDocuments {
        archive: Container,
        documents: Vec<ScannedDocument>,
    },
}

pub trait FileExtractor {
    fn extract(&self, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>>;
}
