use tracing::info;

use crate::engine::{extractor::formats::{DataExtracted, FileExtractor}, scanner::Scanner};

const LOG_TARGET: &str = "extractor_zip";

#[derive(Debug, Clone)]
pub struct ZipExtractor {
    scanner: Scanner
}

impl ZipExtractor {
    pub fn new(scanner: Scanner) -> Self {
        ZipExtractor { scanner }
    }
}

impl FileExtractor for ZipExtractor {
    fn extract(&self, path: &str) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        info!(target: LOG_TARGET, "Extracting files from ZIP archive: {}", path);
        info!(target: LOG_TARGET, "Using scanner: {:?}", self.scanner);

        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let file_path = outpath.to_string_lossy().to_string();

            if self.scanner.check_filters(&outpath) {
                info!(target: LOG_TARGET, "File passed filters: {}", file_path);
            } else {
                //info!(target: LOG_TARGET, "File did not pass filters: {}", file_path);
                continue;
            }
        }

        Ok(DataExtracted::Text(String::new()))
    }
}