use std::thread;

use rsearch::{
    engine::{
        Engine, extractor::Extractor, scanner::{FiltersMode, filters::Filter}, storage::{STORAGE_DATABASE_PATH, StorageEngine}
    },
    init_logging,
};

fn main() {
    init_logging();

    let engine = Engine::new();
    
    let mut extractor = Extractor::new();
    extractor.init(1);
    
    let tx = extractor.get_channel_sender_at(0).expect("Failed to get channel sender");

    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    let mut scanner = engine.scanner;
    scanner.set_channel_sender(tx);

    let t2 = thread::spawn(move || {
        let conn =
            rusqlite::Connection::open(*STORAGE_DATABASE_PATH).expect("Failed to open database");

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

    extractor.join_all().expect("Failed to join extractor workers");
}
