use rsearch::{engine::{scanner::{Scanner, ScannerFilterMode, filters::Filter}, storage::StorageEngine}, entities::document::Document};

fn main() {
    let mut storage = StorageEngine::new();
    storage.initialize().expect("Failed to initialize storage engine");

    let mut doc = Document::new();

    doc.set_filename("file.txt".to_string());
    doc.set_extension(Some("txt".to_string()));
    doc.set_content("This is a test document.".to_string());
    doc.set_description("A simple test document for rsearch.".to_string());
    doc.set_path("/test/file.txt".to_string());

    let conn = storage.get_connection();
    
    if let Err(e) = doc.save(conn) {
        eprintln!("Error saving document: {:?}", e);
    } else {
        println!("Document saved successfully.");
    }

    let mut filter1 = Filter::new();
    filter1.set_case_sensitive(false);
    filter1.set_filename_contains("report");


    let mut filter2 = Filter::new();
    filter2.set_case_sensitive(false);
    filter2.set_filename_contains("bevy");

    let mut scanner = Scanner::new();

    scanner.set_filter_mode(ScannerFilterMode::Or);

    scanner.add_filter(filter1);
    scanner.add_filter(filter2);

    scanner.scan_folder("/home/roothunter/Documents");
    if let Err(e) = scanner.save_documents(&mut storage) {
        eprintln!("Error saving scanned documents: {:?}", e);
    } else {
        println!("Scanned documents saved successfully.");
    }
}