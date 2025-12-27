use crate::engine::extractor::formats::{DataExtracted, FileExtractor};
use crate::entities::document::Document;

use zip::ZipArchive;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;

const LOG_TARGET: &str = "extractor_docx";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocxExtractor;

impl FileExtractor for DocxExtractor {
    fn extract(&self, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
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

        Ok(DataExtracted::Text(text))
    }

    fn extract_compressed(&self, _parent: Document, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        self.extract(document)
    }
}
