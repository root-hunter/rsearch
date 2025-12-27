use std::str::FromStr;

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

impl FromStr for FormatType {
    type Err = ();

    fn from_str(input: &str) -> Result<FormatType, Self::Err> {
        let format = match input.to_lowercase().as_str() {
            "pdf" => FormatType::Pdf,
            "docx" => FormatType::Docx,
            "txt" => FormatType::Text,
            "zip" => FormatType::Archive(Archive::Zip),
            // "tar" => FormatType::Archive(Archive::Tar),
            // "rar" => FormatType::Archive(Archive::Rar),
            // "7z" => FormatType::Archive(Archive::SevenZ),
            _ => FormatType::Unknown,
        };

        Ok(format) 
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
