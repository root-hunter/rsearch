use crate::{
    engine::extractor::formats::{DataExtracted, FileExtractor},
    entities::document::Document,
};

//const LOG_TARGET: &str = "extractor_text";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextExtractor;

impl FileExtractor for TextExtractor {
    fn extract(&self, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(document.get_path())?;
        Ok(DataExtracted::Text(content))
    }

    fn extract_compressed(
        &self,
        _parent: Document,
        document: Document,
    ) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        self.extract(document)
    }
}
