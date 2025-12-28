use std::thread::{self, JoinHandle};

use crossbeam::channel;
use tracing::{info, warn};

use crate::{
    engine::{
        EngineError, PipelineStage, Receiver, Sender,
        extractor::{ExtractorChannelTx, commands::ExtractorCommand},
        scanner::ScannedDocument,
    },
    entities::container::{Container, ContainerType},
    storage::{StorageChannelTx, StorageError, commands::StorageCommand},
};

pub type DecompressorChannelTx = Sender<ScannedDocument>;
pub type DecompressorChannelRx = Receiver<ScannedDocument>;

const LOG_TARGET: &str = "decompressor";

pub struct DecompressorEngine {
    channel_tx: DecompressorChannelTx,
    channel_rx: DecompressorChannelRx,
    channel_extractor_tx: ExtractorChannelTx,
    channel_storage_tx: StorageChannelTx,
}

impl DecompressorEngine {
    pub fn new(
        channel_tx: DecompressorChannelTx,
        channel_rx: DecompressorChannelRx,
        channel_extractor_tx: ExtractorChannelTx,
        channel_storage_tx: StorageChannelTx,
    ) -> Self {
        DecompressorEngine {
            channel_tx,
            channel_rx,
            channel_extractor_tx,
            channel_storage_tx,
        }
    }
}

impl PipelineStage for DecompressorEngine {
    fn add_worker(&mut self) -> Result<JoinHandle<()>, EngineError> {
        info!(target: LOG_TARGET, "Starting decompressor worker");

        let channel_rx = self.channel_rx.clone();
        let channel_extractor_tx = self.channel_extractor_tx.clone();
        let channel_storage_tx = self.channel_storage_tx.clone();

        let handle = thread::spawn(move || {
            while let Ok(scanned) = channel_rx.recv() {
                let container = Container::from_document(&scanned.document, ContainerType::Archive);

                let (tx, rx) = channel::unbounded();

                channel_storage_tx
                    .send(StorageCommand::SaveArchive {
                        archive: container.clone(),
                        resp_tx: Some(tx),
                    })
                    .expect("Failed to send document to storage");

                let _ = rx.recv().expect("Failed to receive storage response");

                let mut documents = Vec::new();

                let file = std::fs::File::open(container.get_path()).map_err(EngineError::IoError);

                if let Err(e) = file {
                    warn!(target: LOG_TARGET, "Failed to open archive file: {:?}", e);
                    continue;
                }

                let file = file.unwrap();

                let archive = zip::ZipArchive::new(file).map_err(EngineError::ZipError);

                if let Err(e) = archive {
                    warn!(target: LOG_TARGET, "Failed to read ZIP archive: {:?}", e);
                    continue;
                }

                let mut archive = archive.unwrap();

                for i in 0..archive.len() {
                    let file = archive.by_index(i).map_err(EngineError::ZipError);

                    if let Err(e) = file {
                        warn!(target: LOG_TARGET, "Failed to read file in ZIP archive: {:?}", e);
                        continue;
                    }

                    let file = file.unwrap();

                    let outpath = match file.enclosed_name() {
                        Some(path) => path.to_owned(),
                        None => continue,
                    };

                    let file_path = outpath.to_string_lossy().to_string();

                    let mut doc = scanned.document.clone();
                    doc.set_filename(file_path);
                    doc.set_status(crate::entities::document::DocumentStatus::Extracted);

                    documents.push(ScannedDocument {
                        container_type: ContainerType::Archive,
                        document: doc,
                    });
                }

                if let Err(e) =
                    channel_extractor_tx.send(ExtractorCommand::ProcessCompressedDocuments {
                        container: container,
                        documents: documents,
                    })
                {
                    info!(target: LOG_TARGET, "Failed to send to extractor: {:?}", e);
                }
            }
        });
        Ok(handle)
    }
}
