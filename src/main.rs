mod engine;
use engine::storage::StorageEngine;
use rsearch::entities::document;

fn main() {
    let storage = StorageEngine::new();
    storage.initialize().expect("Failed to initialize storage engine");

    let mut doc = document::Document::new();

    doc.set_filename("file.txt".to_string());
    doc.set_extension("txt".to_string());
    doc.set_content("This is a test document.".to_string());
    doc.set_description("A simple test document for rsearch.".to_string());
    doc.set_path("/test/file.txt".to_string());

    let conn = storage.get_connection();
    
    if let Err(e) = doc.save(conn) {
        eprintln!("Error saving document: {:?}", e);
    } else {
        println!("Document saved successfully.");
    }
}