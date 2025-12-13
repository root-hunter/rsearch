use rsearch::{engine::{scanner::{Scanner, FiltersMode, filters::Filter}, storage::StorageEngine}, entities::document::Document};

fn main() {
    let mut storage = StorageEngine::new();
    storage.initialize().expect("Failed to initialize storage engine");

    let mut filter1 = Filter::new();
    filter1.set_case_sensitive(false);
    //filter1.set_filename_contains("report");
    filter1.set_extension_is("xlsx");


    let mut filter2 = Filter::new();
    filter2.set_case_sensitive(false);
    filter2.set_filename_contains("bevy");

    let mut scanner = Scanner::new();

    scanner.set_filters_mode(FiltersMode::Or);

    scanner.add_filter(filter1);
    //scanner.add_filter(filter2);

    scanner.scan_folder("/home/roothunter");
    if let Err(e) = scanner.save_documents(&mut storage) {
        eprintln!("Error saving scanned documents: {:?}", e);
    } else {
        println!("Scanned documents saved successfully.");
    }
}