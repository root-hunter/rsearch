use std::thread;

use rsearch::{engine::{
    Engine, extractor::{Extractor, formats::{FormatExtractor, pdf::PdfExtractor}}, scanner::{FiltersMode, filters::Filter}
}, init_logging};

use tracing::{error, info};

fn main() {
    init_logging();

    let extractor = PdfExtractor;
    let t = extractor.extract_text("/home/roothunter/App/NuSMV-2.6.0-Linux/share/nusmv/doc/tutorial.pdf").unwrap();

    info!("Extracted PDF Text:\n{}", t);

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
