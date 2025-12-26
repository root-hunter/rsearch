use tracing::info;

use crate::engine::{extractor::formats::{Archive, ArchiveExtractor}, scanner::Scanner};

pub struct ZipExtractor {
    scanner: Scanner
}

impl ZipExtractor {
    pub fn new(scanner: Scanner) -> Self {
        ZipExtractor { scanner }
    }
}

impl ArchiveExtractor for ZipExtractor {
    fn extract_files(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Extracting files from ZIP archive: {}", path);
        info!("Using scanner: {:?}", self.scanner);

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let file_path = outpath.to_string_lossy().to_string();

            if self.scanner.check_filters(&outpath) {
                info!("File passed filters: {}", file_path);
            } else {
                //info!("File did not pass filters: {}", file_path);
                continue;
            }
        }

        Ok("".into())
    }
}