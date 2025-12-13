use std::thread;

use rsearch::{
    engine::{
        Engine,
        scanner::{FiltersMode, filters::Filter},
        storage::{STORAGE_DATABASE_PATH, StorageEngine},
    },
    init_logging,
};

use tracing::error;

fn main() {
    init_logging();

    let engine = Engine::new();
    let mut extractor = engine.extractor;
    let tx = extractor.get_channel_sender().clone();

    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    let t1 = thread::spawn(move || {
        let mut conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH).expect("Failed to open database");

        if let Err(e) = extractor.process_documents(&mut conn) {
            error!(target: "main", "Error processing documents: {:?}", e);
        }
    });

    threads.push(t1);

    let mut scanner = engine.scanner;
    scanner.set_channel_sender(tx);

    let t2 = thread::spawn(move || {
        let conn = rusqlite::Connection::open(*STORAGE_DATABASE_PATH).expect("Failed to open database");

        StorageEngine::initialize(&conn).expect("Failed to initialize storage engine");

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
        // if let Err(e) = scanner.save_documents(&mut storage) {
        //     error!("Error saving scanned documents: {:?}", e);
        // } else {
        //     info!(target: "main", "Scanned documents saved successfully.");
        // }
    });

    threads.push(t2);

    for handle in threads {
        handle.join().unwrap();
    }
}
