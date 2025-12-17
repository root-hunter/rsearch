pub mod worker;


use crate::{engine::{EngineTask, PipelineStage, Sender, classifier::worker::ClassifierWorker}, entities::document::Document, storage::commands::StorageCommand};

const LOG_TARGET: &str = "classifier";

#[derive(Debug)]
pub struct Classifier {
    database_tx: Sender<StorageCommand>,
    workers: Vec<ClassifierWorker>,
}

impl Classifier {
    pub fn new() -> Self {
        Classifier {
            database_tx: crate::engine::unbounded_channel::<StorageCommand>().0,
            workers: Vec::new(),
        }
    }
}

impl PipelineStage<Document> for Classifier {
    fn get_channel_senders(&self) -> Vec<Sender<Document>> {
        self.workers
            .iter()
            .map(|worker| worker.get_channel_sender().clone())
            .collect()
    }

    fn get_channel_sender_at(&self, index: usize) -> Option<Sender<Document>> {
        self.workers
            .get(index)
            .map(|worker| worker.get_channel_sender().clone())
    }

    fn add_worker(&mut self) {
        
    }
}
