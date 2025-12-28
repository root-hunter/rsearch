use crate::engine::extractor::formats::{FileExtractor};
use crate::engine::extractor::tokens::TextTokensDistribution;
use crate::entities::document::Document;

use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::{BufReader, Read};
use zip::ZipArchive;

//const LOG_TARGET: &str = "extractor_docx";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocxExtractor;

impl FileExtractor for DocxExtractor {
    fn extract(document: Document) -> Result<String, Box<dyn std::error::Error>> {
        let file = File::open(document.get_path())?;
        let mut zip = ZipArchive::new(file)?;

        let mut xml = String::new();
        zip.by_name("word/document.xml")?.read_to_string(&mut xml)?;

        let mut reader = Reader::from_str(&xml);
        //reader.trim_text(true);

        let mut buf = Vec::new();
        let mut text = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Text(e)) => {
                    text.push_str(&e.escape_ascii().to_string());
                    text.push(' ');
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Self::token_distribution(BufReader::new(text.as_bytes()))
    }
}
