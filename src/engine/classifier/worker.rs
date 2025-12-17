use tracing::info;

use crate::{
    engine::{Engine, EngineTask, EngineTaskWorker, Receiver, Sender, unbounded_channel},
    entities::document::Document,
    storage::commands::StorageCommand,
};

const LOG_TARGET: &str = "classifier_worker";

#[derive(Debug)]
pub struct ClassifierWorker {
    id: usize,
    database_tx: Sender<StorageCommand>,
    channel_tx: Sender<Document>,
    channel_rx: Receiver<Document>,
    pub thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl ClassifierWorker {
    pub fn new(id: usize, database_tx: Sender<StorageCommand>) -> Self {
        let (tx, rx) = unbounded_channel::<Document>();

        ClassifierWorker {
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
}

impl EngineTaskWorker<Document> for ClassifierWorker {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl EngineTask<Document> for ClassifierWorker {
    fn name(&self) -> &str {
        LOG_TARGET
    }

    fn get_channel_sender(&self) -> &Sender<Document> {
        &self.channel_tx
    }

    fn get_channel_receiver(&self) -> &Receiver<Document> {
        &self.channel_rx
    }

    fn run(&mut self) {
        let timeout = std::time::Duration::from_millis(200);

        loop {
            if let Ok(document) = self.channel_rx.recv_timeout(timeout) {
                info!(
                    target: LOG_TARGET,
                    id = self.id,
                    "Received document for classification: {:?}",
                    document.get_id()
                );
                // Here would be the classification logic
            } else {
                break; // Exit loop if channel is closed
            }
        }
    }
}
