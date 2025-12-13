pub mod storage;
pub mod scanner;
pub mod classifier;
pub mod utils;

#[derive(Debug)]
pub enum EngineError {
    StorageError(storage::StorageError),
}