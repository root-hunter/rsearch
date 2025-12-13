pub mod storage;
pub mod scanner;

#[derive(Debug)]
pub enum EngineError {
    StorageError(storage::StorageError),
}