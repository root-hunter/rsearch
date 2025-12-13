use std::thread;

use rsearch::engine::{
    Engine,
    scanner::{FiltersMode, filters::Filter},
};

use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub fn init_logging() {
    // file logger (NO ANSI)
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "rsearch");

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

fn main() {
    init_logging();

    let engine = Engine::new();
    let mut storage = engine.storage_engine;
    let mut extractor = engine.extractor;
    let tx = extractor.get_channel_sender().clone();

    thread::spawn(move || {
        loop {
            extractor.process_documents();
        }
    });

    let mut scanner = engine.scanner;
    scanner.set_channel_sender(tx);

    _ = thread::spawn(move || {
        storage
            .initialize()
            .expect("Failed to initialize storage engine");

        let mut filter1 = Filter::new();
        filter1.set_case_sensitive(false);
        //filter1.set_filename_contains("report");
        filter1.set_extension_is("pdf");

        let mut filter2 = Filter::new();
        filter2.set_case_sensitive(false);
        filter2.set_filename_contains("bevy");

        scanner.set_filters_mode(FiltersMode::Or);

        scanner.add_filter(filter1);
        //scanner.add_filter(filter2);

        scanner.scan_folder("/home/roothunter");
        if let Err(e) = scanner.save_documents(&mut storage) {
            error!("Error saving scanned documents: {:?}", e);
        } else {
            info!(target: "main", "Scanned documents saved successfully.");
        }
    })
    .join();
}
