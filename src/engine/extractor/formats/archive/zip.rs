use tracing::info;

use crate::{
    engine::{
        extractor::formats::{DataExtracted, FileExtractor},
        scanner::{ScannedDocument, Scanner},
    },
    entities::{
        container::{Container, ContainerType},
        document::{Document, DocumentStatus},
    },
};

const LOG_TARGET: &str = "extractor_zip";

#[derive(Debug, Clone)]
pub struct ZipExtractor {
    scanner: Scanner,
}

impl ZipExtractor {
    pub fn new(scanner: Scanner) -> Self {
        ZipExtractor { scanner }
    }
}

impl FileExtractor for ZipExtractor {
    fn extract(&self, document: Document) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        info!(target: LOG_TARGET, "Extracting files from ZIP archive: {}", document.get_path());
        info!(target: LOG_TARGET, "Using scanner: {:?}", self.scanner);

        let file = std::fs::File::open(document.get_path())?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut documents = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let file_path = outpath.to_string_lossy().to_string();

            if self.scanner.check_filters(&outpath) {
                info!(target: LOG_TARGET, "File passed filters: {}", file_path);
                let mut doc = Document::from_path(&outpath);
                doc.set_status(DocumentStatus::Extracted);
                doc.set_filename(file_path);

                documents.push(ScannedDocument {
                    container_type: ContainerType::Archive,
                    document: doc,
                });
            } else {
                continue;
            }
        }

        let archive_container = Container::from_document(&document, ContainerType::Archive);

        Ok(DataExtracted::ArchiveDocuments {
            archive: archive_container,
            documents,
        })
    }

    fn extract_compressed(
        &self,
        _parent: Document,
        document: Document,
    ) -> Result<DataExtracted, Box<dyn std::error::Error>> {
        self.extract(document)
    }
}
