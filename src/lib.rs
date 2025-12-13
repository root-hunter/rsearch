pub mod engine;
pub mod entities;

pub const PDFIUM_LIB_PATH: &str = "vendor/pdfium/lib/libpdfium.so";

#[derive(Debug)]
pub enum RSearchError {
    EngineError(engine::EngineError),
    EntityError(entities::EntityError),
}

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_logging() {
    // file logger (NO ANSI)
    let file_appender = RollingFileAppender::new(Rotation::MINUTELY, "logs", "rsearch");

    let file_layer = fmt::layer()
        .with_ansi(false)
        .event_format(
            format()
                .with_level(true)
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false),
        )
        .with_writer(file_appender);

    // console logger (ANSI OK)
    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .event_format(
            format()
                .with_level(true)
                .with_target(true) // 🔥 QUI
                .with_thread_ids(false)
                .with_thread_names(false),
        )
        .with_writer(std::io::stdout);

    let filter = EnvFilter::from_default_env().add_directive("rsearch=info".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();
}