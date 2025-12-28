use std::thread;

use rsearch::{
    engine::{
        EngineTask, PipelineStage,
        classifier::Classifier,
        extractor::Extractor,
        scanner::{FiltersMode, ScannedDocument, Scanner, filters::Filter},
        unbounded_channel,
    },
    init_logging,
    storage::StorageEngine,
};

fn main() {
    init_logging();
    StorageEngine::initialize().expect("Failed to initialize storage engine");

    let (extractor_tx, extractor_rx) = unbounded_channel::<ScannedDocument>();

    let mut storage = StorageEngine::default();
    storage.run();

    let mut scanner = Scanner::new(extractor_tx.clone());
    let mut filter1 = Filter::default();
    filter1.set_case_sensitive(false);
    //filter1.set_filename_contains("report");
    filter1.set_extension_is("zip");

    let mut filter2 = Filter::default();
    filter2.set_extension_is("pdf");

    let mut filter3 = Filter::default();
    filter3.set_extension_is("txt");

    scanner.set_filters_mode(FiltersMode::Or);

    scanner.add_filter(filter1);
    scanner.add_filter(filter2);
    scanner.add_filter(filter3);

    let mut extractor = Extractor::new(
        storage.get_channel_sender().clone(),
        scanner.clone(),
        extractor_tx,
        extractor_rx,
    );
    extractor.init(2);

    let mut classifier = Classifier::default();
    classifier.init(1);

    let _t2 = thread::spawn(move || {
        scanner.scan_folder("/home/roothunter");
    });

    loop {
        thread::sleep(std::time::Duration::from_secs(10));
    }
}
