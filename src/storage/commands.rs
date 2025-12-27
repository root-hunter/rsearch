use crate::{engine::{Sender, scanner::ScannedDocument}, entities::{container::Container, document::Document}, storage::StorageError};

#[derive(Debug, Clone)]
pub enum StorageCommand {
    SaveDocument{
        document: Document,
        resp_tx: Option<Sender<Result<(), StorageError>>>,
    },
    SaveArchive {
        archive: Container,
        resp_tx: Option<Sender<Result<Container, StorageError>>>,
    },
    SaveBulkDocuments{
        documents: Vec<ScannedDocument>,
        resp_tx: Option<Sender<Result<(), StorageError>>>,
    },
}