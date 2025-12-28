pub mod worker;

use std::thread::JoinHandle;

use crate::{
    engine::{EngineError, PipelineStage, Sender, classifier::worker::ClassifierWorker},
    entities::document::Document,
    storage::commands::StorageCommand,
};

//const LOG_TARGET: &str = "classifier";

#[derive(Debug)]
pub struct Classifier {
    database_tx: Sender<StorageCommand>,
    workers: Vec<ClassifierWorker>,
}

impl Default for Classifier {
    fn default() -> Self {
        Classifier {
            database_tx: crate::engine::unbounded_channel::<StorageCommand>().0,
            workers: Vec::new(),
        }
    }
}

impl PipelineStage<Document> for Classifier {
    fn add_worker(&mut self) -> Result<JoinHandle<()>, EngineError> {
        todo!()
    }
}
