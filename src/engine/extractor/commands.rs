use crate::engine::scanner::ScannedDocument;

pub enum ExtractorCommand {
    ProcessDocument(ScannedDocument),
    ProcessCompressedDocuments {
        container: ScannedDocument,
        documents: Vec<ScannedDocument>,
    },
    Flush,
}