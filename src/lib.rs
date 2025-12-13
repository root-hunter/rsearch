pub mod engine;
pub mod entities;

#[derive(Debug)]
pub enum RSearchError {
    EngineError(engine::EngineError),
    EntityError(entities::EntityError),
}