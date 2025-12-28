use std::{io::BufReader, str::FromStr};

use crate::{
    engine::{extractor::tokens::TextTokensDistribution, scanner::ScannedDocument},
    entities::{container::Container, document::Document},
};

pub mod archive;
pub mod microsoft;
pub mod pdf;
pub mod text;

const DEFAULT_MAX_TOKENS: usize = 500;

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
    fn extract(document: Document) -> Result<String, Box<dyn std::error::Error>>;

    fn token_distribution(
        reader: BufReader<impl std::io::Read>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let dist = TextTokensDistribution::from_buffer(reader);
        let content = dist.export_string_nth(DEFAULT_MAX_TOKENS);

        Ok(content)
    }
}
