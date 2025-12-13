use pdfium_render::prelude::*;

use crate::{PDFIUM_LIB_PATH, engine::extractor::formats::FormatExtractor};

pub struct PdfExtractor;

impl FormatExtractor for PdfExtractor {
    fn extract_text(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let lib = if PDFIUM_LIB_PATH.is_empty() {
            Pdfium::bind_to_system_library()?
        } else {
            Pdfium::bind_to_library(PDFIUM_LIB_PATH)?
        };

        let pdfium = Pdfium::new(lib);

        let document = pdfium.load_pdf_from_file(path, None)?;
        let mut text = String::new();

        for page in document.pages().iter() {
            let page_text = page.text().unwrap().to_string();
            text.push_str(&page_text);
            text.push('\n');
        }
        Ok(text)
    }
}