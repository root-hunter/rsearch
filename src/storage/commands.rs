use crate::{engine::{Sender, scanner::ScannedDocument}, entities::document::Document, storage::StorageError};

#[derive(Debug, Clone)]
pub struct CommandSaveDocument {
    pub document: Document,
    pub resp_tx: Option<Sender<Result<(), StorageError>>>,
}

#[derive(Debug, Clone)]
pub struct  CommandSaveBulkDocuments {
    pub documents: Vec<ScannedDocument>,
    pub resp_tx: Option<Sender<Result<(), StorageError>>>,
}

#[derive(Debug, Clone)]
pub enum StorageCommand {
    SaveDocument(CommandSaveDocument),
    SaveBulkDocuments(CommandSaveBulkDocuments),
}