use std::{
    fs::File,
    io::{BufReader},
};

use crate::{
    engine::extractor::{formats::{DataExtracted, FileExtractor}, tokens::TextTokensDistribution},
    entities::document::Document,
};

//const LOG_TARGET: &str = "extractor_text";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextExtractor;

impl FileExtractor for TextExtractor {
    fn extract(document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        let file = File::open(document.get_path())?;
        let reader = BufReader::new(file);

        let dist = TextTokensDistribution::from_buffer(reader);
        let content = dist.export_string_nth(200);

        Ok(DataExtracted::Text(content))
    }
}
