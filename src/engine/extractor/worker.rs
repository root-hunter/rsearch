use std::time::{Duration, Instant};

use crate::{
    engine::{
        ChannelRecvTimeoutError, EngineTask, EngineTaskWorker, Receiver, Sender, extractor::{
            EXTRACTOR_FLUSH_INTERVAL, EXTRACTOR_INSERT_BATCH_SIZE, ExtractorError,
            formats::{self, FormatExtractor, FormatType},
            utils::build_text_content,
        }, unbounded_channel
    },
    entities::document::{Document, DocumentStatus},
    storage::commands::{CommandSaveBulkDocuments, StorageCommand},
};
use tracing::{error, info};

const LOG_TARGET: &str = "extractor_worker";

const WORKER_RECEIVE_TIMEOUT_MS: u64 = 200;

#[derive(Debug)]
pub struct ExtractorWorker {
    id: usize,
    channel_tx: Sender<Document>,
    channel_rx: Receiver<Document>,
    database_tx: Sender<StorageCommand>,
    pub thread_handle: Option<std::thread::JoinHandle<Result<(), ExtractorError>>>,
}

impl ExtractorWorker {
    pub fn new(id: usize, database_tx: Sender<StorageCommand>) -> Self {
        let (tx, rx) = unbounded_channel::<Document>();

        ExtractorWorker {
            id,
            database_tx,
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
        buffer: &mut Vec<Document>,
    ) -> Result<(), ExtractorError> {
        info!(
            target: LOG_TARGET,
            count = buffer.len(),
            "Saving batch"
        );

        let batch = buffer.drain(..).collect::<Vec<_>>();

        database_tx
            .send(StorageCommand::SaveBulkDocuments(
                CommandSaveBulkDocuments {
                    documents: batch,
                    resp_tx: None,
                },
            )).unwrap();
        //Document::save_bulk(conn, batch).map_err(|_| ExtractorError::ExtractionFailed)?;

        Ok(())
    }
}

impl EngineTaskWorker for ExtractorWorker {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl EngineTask<Document> for ExtractorWorker {
    fn get_channel_sender(&self) -> &Sender<Document> {
        &self.channel_tx
    }

    fn get_channel_receiver(&self) -> &Receiver<Document> {
        &self.channel_rx
    }

    fn run(&mut self) {
        assert!(self.thread_handle.is_none(), "Worker is already running");

        let receiver = self.channel_rx.clone();
        let worker_id = self.id;

        let database_tx = self.database_tx.clone();

        self.thread_handle = Some(std::thread::spawn(move || {
            let mut buffer: Vec<Document> = vec![];
            let mut last_flush = Instant::now();

            loop {
                match receiver.recv_timeout(Duration::from_millis(WORKER_RECEIVE_TIMEOUT_MS)) {
                    Ok(mut document) => {
                        info!(target: LOG_TARGET, worker_id = worker_id, "Processing document: {:?}", document);

                        match document.get_format_type() {
                            FormatType::Pdf => {
                                let extractor = formats::pdf::PdfExtractor;
                                match extractor.extract_text(document.get_path()) {
                                    Ok(text) => {
                                        info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text from PDF, length: {}", text.len());

                                        let content = build_text_content(text);

                                        document.set_content(content);
                                        document.set_status(DocumentStatus::Extracted);

                                        buffer.push(document);
                                    }
                                    Err(e) => {
                                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract text from PDF: {:?} ({})", e, document.get_path());
                                    }
                                }
                            }
                            FormatType::Txt => {
                                error!(target: LOG_TARGET, worker_id = worker_id, "Text extraction not implemented yet.");
                            }
                            FormatType::Docx => {
                                let extractor = formats::microsoft::docx::DocxExtractor;
                                match extractor.extract_text(document.get_path()) {
                                    Ok(text) => {
                                        info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text from DOCX, length: {}", text.len());

                                        let content: String = build_text_content(text);
                                        info!(target: LOG_TARGET, "Extracted text distribution: {:?}", content);

                                        document.set_content(content);
                                        document.set_status(DocumentStatus::Extracted);

                                        buffer.push(document);
                                    }
                                    Err(e) => {
                                        error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract text from DOCX: {:?} ({})", e, document.get_path());
                                    }
                                }
                            }
                            _ => {
                                error!(target: LOG_TARGET, worker_id = worker_id, "Unsupported document format: {:?}", document.get_format_type());
                            }
                        }
                    }
                    Err(ChannelRecvTimeoutError::Timeout) => {
                        // Timeout occurred, check if we need to flush
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
