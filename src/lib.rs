pub mod api;
pub mod engine;
pub mod entities;
pub mod storage;

#[derive(Debug)]
pub enum RSearchError {
    EngineError(engine::EngineError),
    EntityError(entities::EntityError),
}

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_logging() {
    // file logger (NO ANSI)
    let file_appender = RollingFileAppender::new(Rotation::HOURLY, "logs", "rsearch");

    let file_layer = fmt::layer()
        .with_ansi(false)
        .event_format(
            format()
                .with_level(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with_writer(file_appender);

    // console logger (ANSI OK)
    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .event_format(
            format()
                .with_level(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();
}
