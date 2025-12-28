use std::{fs::File, io::BufReader};

use crate::{engine::extractor::formats::FileExtractor, entities::document::Document};

//const LOG_TARGET: &str = "extractor_text";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextExtractor;

impl FileExtractor for TextExtractor {
    fn extract(document: Document) -> Result<String, Box<dyn std::error::Error>> {
        let file = File::open(document.get_path())?;

        Self::token_distribution(BufReader::new(file))
    }
}
