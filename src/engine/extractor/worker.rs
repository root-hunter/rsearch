use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    engine::{
        ChannelRecvTimeoutError, EngineTask, EngineTaskWorker, Receiver, Sender,
        extractor::{
            EXTRACTOR_FLUSH_INTERVAL, EXTRACTOR_INSERT_BATCH_SIZE, ExtractorError,
            formats::{
                self, DataExtracted, FileExtractor, FormatType, archive::zip::ZipExtractor,
                microsoft::docx::DocxExtractor, pdf::PdfExtractor, text::TextExtractor,
            },
        },
        scanner::{ScannedDocument, Scanner},
        unbounded_channel,
    },
    entities::{
        container::{Container, ContainerType},
        document::DocumentStatus,
    },
    storage::{StorageError, commands::StorageCommand},
};
use tracing::{error, info};

const LOG_TARGET: &str = "extractor_worker";

const WORKER_RECEIVE_TIMEOUT_MS: u64 = 200;

#[derive(Debug)]
pub struct ExtractorWorker {
    id: usize,
    channel_tx: Sender<ScannedDocument>,
    channel_rx: Receiver<ScannedDocument>,
    database_tx: Sender<StorageCommand>,
    scanner: Scanner,
    pub thread_handle: Option<std::thread::JoinHandle<Result<(), ExtractorError>>>,
}

impl ExtractorWorker {
    pub fn new(id: usize, database_tx: Sender<StorageCommand>, scanner: Scanner) -> Self {
        let (tx, rx) = unbounded_channel::<ScannedDocument>();

        ExtractorWorker {
            id,
            database_tx,
            scanner,
            channel_tx: tx,
            channel_rx: rx,
            thread_handle: None,
        }
    }

    pub fn get_database_tx(&self) -> &Sender<StorageCommand> {
        &self.database_tx
    }

    pub fn flush_buffer(
        database_tx: Sender<StorageCommand>,
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

impl EngineTaskWorker<ScannedDocument> for ExtractorWorker {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl EngineTask<ScannedDocument> for ExtractorWorker {
    fn name(&self) -> &str {
        LOG_TARGET
    }

    fn get_channel_sender(&self) -> &Sender<ScannedDocument> {
        &self.channel_tx
    }

    fn get_channel_receiver(&self) -> &Receiver<ScannedDocument> {
        &self.channel_rx
    }

    fn run(&mut self) {
        assert!(self.thread_handle.is_none(), "Worker is already running");

        let receiver = self.channel_rx.clone();
        let worker_id = self.id;

        let database_tx = self.database_tx.clone();
        let scanner = self.scanner.clone();
        let channel_tx = self.channel_tx.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            let mut buffer: Vec<ScannedDocument> = vec![];
            let mut last_flush = Instant::now();

            loop {
                match receiver.recv_timeout(Duration::from_millis(WORKER_RECEIVE_TIMEOUT_MS)) {
                    Ok(mut scanned) => {
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
                                if let Ok(data) = PdfExtractor::extract(document.clone()) {
                                    match data {
                                        DataExtracted::Text(content) => {
                                            info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", content.len());

                                            document.set_content(content);
                                            document.set_status(DocumentStatus::Extracted);

                                            buffer.push(scanned);
                                        }
                                        _ => {}
                                    }
                                } else {
                                    error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract PDF document: {:?}", document);
                                }
                            }
                            FormatType::Docx => {
                                if let Ok(data) = DocxExtractor::extract(document.clone()) {
                                    match data {
                                        DataExtracted::Text(content) => {
                                            info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", content.len());

                                            document.set_content(content);
                                            document.set_status(DocumentStatus::Extracted);

                                            buffer.push(scanned);
                                        }
                                        _ => {}
                                    }
                                } else {
                                    error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract DOCX document: {:?}", document);
                                }
                            }
                            FormatType::Text => {
                                if let Ok(data) = TextExtractor::extract(document.clone()) {
                                    match data {
                                        DataExtracted::Text(content) => {
                                            info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", content.len());

                                            document.set_content(content);
                                            document.set_status(DocumentStatus::Extracted);

                                            buffer.push(scanned);
                                        }
                                        _ => {}
                                    }
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
                                                                channel_tx
                                                                    .send(ScannedDocument {
                                                                        container_type: scanned_doc
                                                                            .container_type,
                                                                        document: doc,
                                                                    })
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
                    Err(ChannelRecvTimeoutError::Timeout) => {
                        // Check if we need to flush the buffer
                    }
                    Err(e) => {
                        error!(target: LOG_TARGET, "Channel receive error: {:?}", e);
                        break;
                    }
                }

                if buffer.len() >= *EXTRACTOR_INSERT_BATCH_SIZE {
                    Self::flush_buffer(database_tx.clone(), &mut buffer)?;
                    last_flush = Instant::now();
                }

                if !buffer.is_empty() && last_flush.elapsed() >= *EXTRACTOR_FLUSH_INTERVAL {
                    Self::flush_buffer(database_tx.clone(), &mut buffer)?;
                    last_flush = Instant::now();
                }
            }
            Ok(())
        }));
    }
}
