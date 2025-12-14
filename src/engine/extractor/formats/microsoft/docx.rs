use crate::engine::extractor::formats::FormatExtractor;

use zip::ZipArchive;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fs::File;
use std::io::Read;

pub struct DocxExtractor;

impl FormatExtractor for DocxExtractor {
    fn extract_text(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
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

        Ok(text)
    }
}
