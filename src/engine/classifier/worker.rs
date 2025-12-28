use std::thread::JoinHandle;

use tracing::info;

use crate::{
    engine::{EngineError, EngineTask, EngineTaskWorker, Receiver, Sender, unbounded_channel},
    entities::document::Document,
    storage::commands::StorageCommand,
};

const LOG_TARGET: &str = "classifier_worker";

pub type ClassifierChannelTx = Sender<Document>;
pub type ClassifierChannelRx = Receiver<Document>;

#[derive(Debug)]
pub struct ClassifierWorker {
    id: usize,
    database_tx: Sender<StorageCommand>,
    channel_tx: Sender<Document>,
    channel_rx: Receiver<Document>,
}

impl ClassifierWorker {
    pub fn new(id: usize, database_tx: Sender<StorageCommand>) -> Self {
        let (tx, rx) = unbounded_channel::<Document>();

        ClassifierWorker {
            id,
            database_tx,
            channel_tx: tx,
            channel_rx: rx,
        }
    }

    pub fn get_database_tx(&self) -> &Sender<StorageCommand> {
        &self.database_tx
    }
}

impl EngineTaskWorker<ClassifierChannelTx, ClassifierChannelRx> for ClassifierWorker {
    fn get_id(&self) -> usize {
        self.id
    }
}

impl EngineTask<ClassifierChannelTx, ClassifierChannelRx> for ClassifierWorker {
    fn name(&self) -> &str {
        LOG_TARGET
    }

    fn get_channel_tx(&self) -> &ClassifierChannelTx {
        &self.channel_tx
    }

    fn get_channel_rx(&self) -> &ClassifierChannelRx {
        &self.channel_rx
    }

    fn run(&mut self) -> Result<JoinHandle<()>, EngineError> {
        let timeout = std::time::Duration::from_millis(200);

        while let Ok(document) = self.channel_rx.recv_timeout(timeout) {
            info!(
            target: LOG_TARGET,
            id = self.id,
            "Received document for classification: {:?}",
                document.get_id()
            );
        }

        todo!()
    }
}
