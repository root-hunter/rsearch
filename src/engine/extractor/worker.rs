use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    engine::{
        ChannelRecvTimeoutError, EngineTask, EngineTaskWorker, Receiver, Sender,
        extractor::{
            EXTRACTOR_FLUSH_INTERVAL, EXTRACTOR_INSERT_BATCH_SIZE, ExtractorError,
            formats::{self, DataExtracted, FileExtractor, FormatType},
            utils::build_text_content,
        },
        scanner::{ScannedDocument, Scanner},
        unbounded_channel,
    },
    entities::document::DocumentStatus,
    storage::commands::{CommandSaveBulkDocuments, StorageCommand},
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
            channel_tx: tx,
            channel_rx: rx,
            scanner: scanner,
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

        let batch = buffer.drain(..).collect::<Vec<_>>();

        database_tx
            .send(StorageCommand::SaveBulkDocuments(
                CommandSaveBulkDocuments {
                    documents: batch,
                    resp_tx: None,
                },
            ))
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

        self.thread_handle = Some(std::thread::spawn(move || {
            let mut buffer: Vec<ScannedDocument> = vec![];
            let mut last_flush = Instant::now();

            let mut extractors_map: HashMap<FormatType, Box<dyn FileExtractor>> = HashMap::new();

            extractors_map.insert(FormatType::Text, Box::new(formats::text::TextExtractor));
            extractors_map.insert(FormatType::Pdf, Box::new(formats::pdf::PdfExtractor));
            extractors_map.insert(
                FormatType::Docx,
                Box::new(formats::microsoft::docx::DocxExtractor),
            );
            extractors_map.insert(
                FormatType::Archive(formats::Archive::Zip),
                Box::new(formats::archive::zip::ZipExtractor::new(scanner.clone())),
            );

            loop {
                match receiver.recv_timeout(Duration::from_millis(WORKER_RECEIVE_TIMEOUT_MS)) {
                    Ok(mut scanned) => {
                        info!(target: LOG_TARGET, worker_id = worker_id, "Processing document: {:?}", scanned);

                        let document = &mut scanned.document;
                        let document_format = document.get_format_type();

                        let extractor = extractors_map.get(&document_format);

                        if let Some(extractor) = extractor {
                            match extractor.extract(document.get_path()) {
                                Ok(data_extracted) => match data_extracted {
                                    DataExtracted::Text(text) => {
                                        info!(target: LOG_TARGET, worker_id = worker_id, "Extracted text, length: {}", text.len());

                                        let content = build_text_content(text);

                                        document.set_content(content);
                                        document.set_status(DocumentStatus::Extracted);

                                        buffer.push(scanned);
                                    }
                                    _ => {
                                        error!(target: LOG_TARGET, worker_id = worker_id, "Unsupported extracted data type for document: {:?}", document);
                                    }
                                },
                                Err(e) => {
                                    error!(target: LOG_TARGET, worker_id = worker_id, "Failed to extract text: {:?} ({})", e, document.get_path());
                                }
                            }
                        } else {
                            error!(target: LOG_TARGET, worker_id = worker_id, "No extractor found for document format: {:?}", document_format);
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
