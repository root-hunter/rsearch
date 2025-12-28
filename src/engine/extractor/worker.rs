use std::{
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crate::{
    engine::{
        ChannelRecvTimeoutError, EngineError, EngineTask, EngineTaskWorker, Receiver, Sender,
        extractor::{
            EXTRACTOR_FLUSH_INTERVAL, EXTRACTOR_INSERT_BATCH_SIZE, ExtractorChannelRx, ExtractorChannelTx, ExtractorCommand, ExtractorError, formats::{
                self, DataExtracted, FileExtractor, FormatType, archive::zip::ZipExtractor,
                microsoft::docx::DocxExtractor, pdf::PdfExtractor, text::TextExtractor,
            }
        },
        scanner::{ScannedDocument, Scanner},
        unbounded_channel,
    },
    entities::{
        container::{Container, ContainerType},
        document::DocumentStatus,
    },
    storage::{StorageChannelTx, StorageError, commands::StorageCommand},
};
use tracing::{error, info, warn};

const LOG_TARGET: &str = "extractor_worker";

const WORKER_RECEIVE_TIMEOUT_MS: u64 = 200;

#[derive(Debug)]
pub struct ExtractorWorker {
    id: usize,
    channel_tx: ExtractorChannelTx,
    channel_rx: ExtractorChannelRx,
    database_tx: StorageChannelTx,
    scanner: Scanner,
}

impl ExtractorWorker {
    pub fn new(
        id: usize,
        database_tx: StorageChannelTx,
        scanner: Scanner,
        channel_tx: ExtractorChannelTx,
        channel_rx: ExtractorChannelRx,
    ) -> Self {
        ExtractorWorker {
            id,
            database_tx,
            scanner,
            channel_tx,
            channel_rx,
        }
    }

    pub fn get_database_tx(&self) -> &StorageChannelTx {
        &self.database_tx
    }

    pub fn flush_buffer(
        database_tx: StorageChannelTx,
        buffer: &mut Vec<ScannedDocument>,
    ) -> Result<(), ExtractorError> {
        info!(
            target: LOG_TARGET,
            count = buffer.len(),
            "Saving batch"
        );

        //let batch = buffer.drain(..).collect::<Vec<_>>();
        let batch = std::mem::take(buffer);

        database_tx
            .send(StorageCommand::SaveBulkDocuments {
                documents: batch,
                resp_tx: None,
            })
            .unwrap();
        //Document::save_bulk(conn, batch).map_err(|_| ExtractorError::ExtractionFailed)?;

        Ok(())
    }
}

impl EngineTaskWorker<ExtractorChannelTx, ExtractorChannelRx> for ExtractorWorker {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl EngineTask<ExtractorChannelTx, ExtractorChannelRx> for ExtractorWorker {
    fn name(&self) -> &str {
        LOG_TARGET
    }

    fn get_channel_tx(&self) -> &ExtractorChannelTx {
        &self.channel_tx
    }

    fn get_channel_rx(&self) -> &ExtractorChannelRx {
        &self.channel_rx
    }

    fn run(&mut self) -> Result<JoinHandle<()>, EngineError> {
        let receiver = self.channel_rx.clone();
        let worker_id = self.id;

        let database_tx = self.database_tx.clone();
        let scanner = self.scanner.clone();
        let channel_tx = self.channel_tx.clone();

        let handle = std::thread::spawn(move || {
            let mut buffer: Vec<ScannedDocument> = vec![];
            let mut last_flush = Instant::now();

            loop {
                match receiver.recv_timeout(Duration::from_millis(WORKER_RECEIVE_TIMEOUT_MS)) {
                    Ok(mut command) => {
                        match command {
                            ExtractorCommand::ProcessDocument(mut scanned) => {
                                                        info!(target: LOG_TARGET, worker_id = worker_id, "Processing document: {:?}", scanned);

                        let document = &mut scanned.document;
                        let document_format = document.get_format_type();

                        if scanned.container_type == ContainerType::Archive {
                            info!(target: LOG_TARGET, worker_id = worker_id, "Extracting from archive document: {:?}", document);
                            document.set_status(DocumentStatus::Extracted);

                            buffer.push(scanned);
                            continue;
                        }

                        match document_format {
                            FormatType::Pdf => {
                                if let Ok(content) = PdfExtractor::extract(document.clone()) {
                                    info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", content.len());

                                    document.set_content(content);
                                    document.set_status(DocumentStatus::Extracted);

                                    buffer.push(scanned);
                                } else {
                                    error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract PDF document: {:?}", document);
                                }
                            }
                            FormatType::Docx => {
                                if let Ok(content) = DocxExtractor::extract(document.clone()) {
                                    info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", content.len());

                                    document.set_content(content);
                                    document.set_status(DocumentStatus::Extracted);

                                    buffer.push(scanned);
                                } else {
                                    error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract DOCX document: {:?}", document);
                                }
                            }
                            FormatType::Text => {
                                if let Ok(content) = TextExtractor::extract(document.clone()) {
                                    info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", content.len());

                                    document.set_content(content);
                                    document.set_status(DocumentStatus::Extracted);

                                    buffer.push(scanned);
                                } else {
                                    error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract TEXT document: {:?}", document);
                                }
                            }
                            FormatType::Archive(archive) => match archive {
                                formats::Archive::Zip => {
                                    let zip_extractor = ZipExtractor::new(scanner.clone());
                                    match zip_extractor.extract(document.clone()) {
                                        Ok(data) => match data {
                                            DataExtracted::ArchiveDocuments {
                                                archive,
                                                documents,
                                            } => {
                                                info!(target: LOG_TARGET, worker_id = worker_id, "Extracted {} documents from ZIP archive", documents.len());

                                                let (resp_tx, resp_rx) = unbounded_channel::<
                                                    Result<Container, StorageError>,
                                                >(
                                                );

                                                database_tx
                                                    .send(StorageCommand::SaveArchive {
                                                        archive,
                                                        resp_tx: Some(resp_tx),
                                                    })
                                                    .unwrap();

                                                match resp_rx.recv() {
                                                    Ok(result) => match result {
                                                        Ok(archive) => {
                                                            info!(target: LOG_TARGET, worker_id = worker_id, "Archive saved successfully with ID: {}", archive.get_id());

                                                            for scanned_doc in documents {
                                                                let mut doc = scanned_doc.document;
                                                                doc.set_container_id(
                                                                    archive.get_id(),
                                                                );

                                                                let document = ExtractorCommand::ProcessDocument(ScannedDocument {
                                                                    container_type: scanned_doc
                                                                        .container_type,
                                                                    document: doc.clone(),
                                                                });

                                                                channel_tx
                                                                    .send(document)
                                                                    .unwrap();
                                                            }
                                                        }
                                                        Err(e) => {
                                                            error!(target: LOG_TARGET, worker_id = worker_id, "Failed to save archive: {:?}", e);
                                                        }
                                                    },
                                                    Err(e) => {
                                                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to receive archive save response: {:?}", e);
                                                    }
                                                }
                                            }
                                            _ => {}
                                        },
                                        Err(e) => {
                                            error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract ZIP archive: {:?} ({})", e, document.get_path());
                                        }
                                    }
                                }
                            },
                            FormatType::Unknown => {
                                error!(target: LOG_TARGET, worker_id = worker_id, "Unknown document format for document: {:?}", document);
                                continue;
                            }
                        }

                            }
                            _ => {
                                warn!(target: LOG_TARGET, worker_id = worker_id, "Received unsupported command");
                                continue;
                            }
                        }
                    }
                    Err(ChannelRecvTimeoutError::Timeout) => {
                        // Check if we need to flush the buffer
                    }
                    Err(e) => {
                        error!(target: LOG_TARGET, "Channel receive error: {:?}", e);
                        break;
                    }
                }

                if buffer.len() >= *EXTRACTOR_INSERT_BATCH_SIZE {
                    if let Err(e) = Self::flush_buffer(database_tx.clone(), &mut buffer) {
                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to flush buffer: {:?}", e);
                    }
                    last_flush = Instant::now();
                }

                if !buffer.is_empty() && last_flush.elapsed() >= *EXTRACTOR_FLUSH_INTERVAL {
                    if let Err(e) = Self::flush_buffer(database_tx.clone(), &mut buffer) {
                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to flush buffer: {:?}", e);
                    }
                    last_flush = Instant::now();
                }
            }
        });

        Ok(handle)
    }
}
