use std::thread;

use rsearch::{
    engine::{
        extractor::Extractor, scanner::{FiltersMode, Scanner, filters::Filter}, storage::{STORAGE_DATABASE_PATH, StorageEngine}
    },
    init_logging,
};

fn main() {
    init_logging();

    let mut extractor = Extractor::new();
    extractor.init(2);

    let channels = extractor.get_channel_senders();
    
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    let mut scanner = Scanner::new();
    scanner.add_channel_senders(channels);

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

    loop {
        
    }
}
