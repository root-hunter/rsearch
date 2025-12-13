use crate::entities::document::Document;
use crossbeam::channel;

#[derive(Debug, Clone)]
pub enum ExtractorType {
    Pdf,
    Docx,
    Txt,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Extractor {
    extractor_type: ExtractorType,
    channel_tx: crossbeam::channel::Sender<Document>,
    channel_rx: crossbeam::channel::Receiver<Document>,
}

impl Extractor {
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded::<Document>();

        Extractor {
            channel_tx: tx,
            channel_rx: rx,
            extractor_type: ExtractorType::Unknown,
        }
    }

    pub fn set_extractor_type(&mut self, extractor_type: ExtractorType) {
        self.extractor_type = extractor_type;
    }

    pub fn get_extractor_type(&self) -> &ExtractorType {
        &self.extractor_type
    }

    pub fn get_channel_sender(&self) -> &channel::Sender<Document> {
        &self.channel_tx
    }

    pub fn get_channel_receiver(&self) -> &channel::Receiver<Document> {
        &self.channel_rx
    }

    pub fn extract(&self, data: &str) {
        // Extraction logic would go here
        println!("Extracting data: {}", data);
    }

    pub fn process_documents(&self) {
        while let Ok(doc) = self.channel_rx.try_recv() {
            println!("Processing document: {:?}", doc);
            // Further processing logic would go here
        }
    }
}