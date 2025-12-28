use rsearch::{
    engine::{
        EngineTask, PipelineStage,
        extractor::{Extractor, commands::ExtractorCommand},
        scanner::{FiltersMode, Scanner, filters::Filter},
        unbounded_channel,
    },
    init_logging,
    storage::StorageEngine,
};
use tracing::warn;

fn main() {
    init_logging();
    StorageEngine::initialize().expect("Failed to initialize storage engine");

    let (scanner_tx, scanner_rx) = unbounded_channel::<String>();
    let (extractor_tx, extractor_rx) = unbounded_channel::<ExtractorCommand>();

    let mut storage = StorageEngine::default();
    let storage_handle = storage.run();

    let mut scanner = Scanner::new(scanner_tx.clone(), scanner_rx, extractor_tx.clone());
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
        storage.get_channel_tx().clone(),
        scanner.clone(),
        extractor_tx,
        extractor_rx,
    );

    let scanner_handle = scanner.init().expect("Failed to start scanner");
    let extractor_handles = extractor.init(16).expect("Failed to initialize extractor");

    // let mut classifier = Classifier::default();
    // let classifier_handles = classifier.init(1).expect("Failed to initialize classifier");

    let api = rsearch::api::Api::new(scanner_tx.clone());

    api.scan_path("/home/roothunter/Documents".to_string())
        .expect("Failed to send scan command");

    for handle in scanner_handle {
        handle.join().expect("Scanner thread panicked");
        warn!(target: "main", "Scanner thread has finished");
    }

    for handle in extractor_handles {
        handle.join().expect("Extractor thread panicked");
        warn!(target: "main", "Extractor thread has finished");
    }

    if let Ok(handle) = storage_handle {
        handle.join().expect("Storage thread panicked");
        warn!(target: "main", "Storage thread has finished");
    }
}
