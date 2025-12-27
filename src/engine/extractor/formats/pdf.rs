use std::{env, io::BufReader};

use once_cell::sync::Lazy;
use pdfium_render::prelude::*;

use crate::{
    engine::extractor::{formats::{DataExtracted, FileExtractor}, tokens::TextTokensDistribution},
    entities::document::Document,
};

//const LOG_TARGET: &str = "extractor_pdf";

pub static PDFIUM_LIB_PATH: Lazy<&'static str> = Lazy::new(|| {
    Box::leak(
        env::var("PDFIUM_LIB_PATH")
            .unwrap_or_else(|_| "vendor/pdfium/lib/libpdfium.so".into())
            .into_boxed_str(),
    )
});

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdfExtractor;

impl FileExtractor for PdfExtractor {
    fn extract(&self, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        let lib = if PDFIUM_LIB_PATH.is_empty() {
            Pdfium::bind_to_system_library()?
        } else {
            Pdfium::bind_to_library(*PDFIUM_LIB_PATH)?
        };

        let pdfium = Pdfium::new(lib);

        let document = pdfium.load_pdf_from_file(document.get_path(), None)?;
        let mut text = String::new();

        for page in document.pages().iter() {
            let page_text = page.text().unwrap().to_string();
            text.push_str(&page_text);
            text.push('\n');
        }

        let reader = BufReader::new(text.as_bytes());
        let dist = TextTokensDistribution::from_buffer(reader);
        let text = dist.export_string_nth(500);

        Ok(DataExtracted::Text(text))
    }
}
