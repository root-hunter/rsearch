use std::fs as fs;

fn main() {
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open("app.log").expect("Failed to open log file");

    file.metadata().expect("Failed to get metadata");
}