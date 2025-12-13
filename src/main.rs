use std::thread;

use rsearch::engine::{
    Engine,
    extractor::{self, Extractor},
    scanner::{FiltersMode, Scanner, filters::Filter},
    storage::StorageEngine,
};

fn main() {
    let engine = Engine::new();
    let mut storage = engine.storage_engine;
    let mut extractor = engine.extractor;

    thread::spawn(move || {
        loop {
            extractor.process_documents();
        }
    });

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

        let mut scanner = engine.scanner;

        scanner.set_filters_mode(FiltersMode::Or);

        scanner.add_filter(filter1);
        //scanner.add_filter(filter2);

        scanner.scan_folder("/home/roothunter");
        if let Err(e) = scanner.save_documents(&mut storage) {
            eprintln!("Error saving scanned documents: {:?}", e);
        } else {
            println!("Scanned documents saved successfully.");
        }
    }).join();
}
