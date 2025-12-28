use crate::{engine::scanner::ScannedDocument, entities::container::Container};

pub enum ExtractorCommand {
    ProcessDocument(ScannedDocument),
    ProcessCompressedDocuments {
        container: Container,
        documents: Vec<ScannedDocument>,
    },
}