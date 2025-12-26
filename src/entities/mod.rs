pub mod container;
pub mod document;

#[derive(Debug)]
pub enum EntityError {
    DocumentError(document::DocumentError),
}