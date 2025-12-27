use std::thread;

use rsearch::{
    engine::{
        EngineTask, PipelineStage,
        classifier::Classifier,
        extractor::Extractor,
        scanner::{FiltersMode, Scanner, filters::Filter},
    },
    init_logging,
    storage::StorageEngine,
};

fn main() {
    init_logging();
    StorageEngine::initialize().expect("Failed to initialize storage engine");

    let mut storage = StorageEngine::default();
    storage.run();

    let mut scanner = Scanner::default();
    let mut filter1 = Filter::default();
    filter1.set_case_sensitive(false);
    //filter1.set_filename_contains("report");
    filter1.set_extension_is("zip");

    let mut filter2 = Filter::default();
    filter2.set_extension_is("pdf");

    scanner.set_filters_mode(FiltersMode::Or);

    scanner.add_filter(filter1);
    scanner.add_filter(filter2);

    let mut extractor = Extractor::new(storage.get_channel_sender().clone(), scanner.clone());
    extractor.init(16);

    let mut classifier = Classifier::default();
    classifier.init(2);

    let channels = extractor.get_channel_senders();

    scanner.add_channel_senders(channels);

    let _t2 = thread::spawn(move || {
        scanner.scan_folder("/home/roothunter/Documents");
    });

    loop {}
}
