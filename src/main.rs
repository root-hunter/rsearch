use rsearch::{engine::{scanner::{Scanner, filters::Filter}, storage::StorageEngine}, entities::document::Document};

fn main() {
    let storage = StorageEngine::new();
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

    let mut filter = Filter::new();
    filter.set_case_sensitive(false);
    filter.set_filename_contains("report");

    let mut scanner = Scanner::new();

    scanner.add_filter(filter);
    scanner.scan_folder("/home/roothunter/Documents");
    scanner.save_documents(&storage);
}