use std::thread;

use rsearch::{
    engine::{
        EngineTask, PipelineStage, classifier::Classifier, extractor::Extractor, scanner::{FiltersMode, Scanner, filters::Filter}
    },
    init_logging,
    storage::StorageEngine,
};

fn main() {
    init_logging();
    StorageEngine::initialize().expect("Failed to initialize storage engine");

    let mut storage = StorageEngine::new();
    storage.run();

    let mut extractor = Extractor::new(storage.get_channel_sender().clone());
    extractor.init(4);

    let mut classifier  = Classifier::new();
    classifier.init(2);

    let channels = extractor.get_channel_senders();

    let mut scanner = Scanner::new();
    scanner.add_channel_senders(channels);

    let _t2 = thread::spawn(move || {
        let mut filter1 = Filter::new();
        filter1.set_case_sensitive(false);
        //filter1.set_filename_contains("report");
        filter1.set_extension_is("txt");

        let mut filter2 = Filter::new();
        filter2.set_extension_is("docx");

        scanner.set_filters_mode(FiltersMode::Or);

        scanner.add_filter(filter1);
        //scanner.add_filter(filter2);

        scanner.scan_folder("/home/roothunter/Documents");
        // if let Err(e) = scanner.save_documents(&mut storage) {
        //     error!("Error saving scanned documents: {:?}", e);
        // } else {
        //     info!(target: "main", "Scanned documents saved successfully.");
        // }
    });

    loop {}
}
