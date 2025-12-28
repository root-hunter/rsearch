use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};

use tracing::info;

use crate::{
    engine::{extractor::formats::FormatType, scanner::ScannedDocument},
    entities::container::Container,
};

const LOG_TARGET: &str = "document";

#[derive(Debug)]
pub enum DocumentError {
    NotFound,
    ConstraintViolation,
    DatabaseError(rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentStatus {
    New,
    Scanned,
    Extracted,
    Classified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Document {
    id: Option<i64>,
    path: String,
    filename: String,
    extension: Option<String>,
    content: String,
    description: String,
    status: DocumentStatus,
    container_id: Option<i64>,
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Document {{ id: {:?}, path: {}, filename: {}, extension: {:?}, status: {:?}, container_id: {:?} }}",
            self.id, self.path, self.filename, self.extension, self.status, self.container_id
        )
    }
}

impl Default for Document {
    fn default() -> Self {
        Document {
            id: None,
            path: String::new(),
            filename: String::new(),
            extension: None,
            content: String::new(),
            description: String::new(),
            status: DocumentStatus::New,
            container_id: None,
        }
    }
}

impl Document {
    pub fn from_path(path: &Path) -> Self {
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        let extension = if extension.is_empty() {
            None
        } else {
            Some(extension)
        };

        Document {
            id: None,
            path: path.to_string_lossy().to_string(),
            filename,
            extension,
            content: String::new(),
            description: String::new(),
            status: DocumentStatus::New,
            container_id: None,
        }
    }

    pub fn set_path(&mut self, path: String) {
        self.path = path;
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn set_filename(&mut self, filename: String) {
        self.filename = filename;
    }

    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    pub fn set_extension(&mut self, extension: Option<String>) {
        self.extension = extension;
    }

    pub fn get_extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_id(&self) -> Option<i64> {
        self.id
    }

    pub fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    pub fn set_container_id(&mut self, container_id: i64) {
        self.container_id = Some(container_id);
    }

    pub fn get_container_id(&self) -> Option<i64> {
        self.container_id
    }

    pub fn save(&mut self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        conn.execute(
            "INSERT INTO documents (path, filename, extension, status, container_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![self.path, self.filename, self.extension, self.get_status_str(), self.container_id],
        )
        .map_err(|err| {
            if let rusqlite::Error::SqliteFailure(ref err_code, _) = err && err_code.code == rusqlite::ErrorCode::ConstraintViolation {
                return DocumentError::ConstraintViolation;
            }
            DocumentError::DatabaseError(err)
        })?;

        let document_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO index_documents (document_id, content, description) VALUES (?1, ?2, ?3)",
            rusqlite::params![document_id, self.content, self.description],
        )
        .map_err(DocumentError::DatabaseError)?;

        self.set_id(document_id);

        Ok(())
    }

    pub fn get_id_by_path(&self, conn: &rusqlite::Connection) -> Result<i64, DocumentError> {
        let document_id: i64 = conn
            .query_row(
                "SELECT id FROM documents_view WHERE path = ?1",
                rusqlite::params![self.path],
                |row| row.get(0),
            )
            .map_err(|err| {
                if err == rusqlite::Error::QueryReturnedNoRows {
                    DocumentError::NotFound
                } else {
                    DocumentError::DatabaseError(err)
                }
            })?;

        Ok(document_id)
    }

    pub fn _get_id(&self, conn: &rusqlite::Connection) -> Result<i64, DocumentError> {
        if let Some(id) = self.id {
            Ok(id)
        } else {
            self.get_id_by_path(conn)
        }
    }

    pub fn save_bulk(
        conn: &mut rusqlite::Connection,
        documents: Vec<ScannedDocument>,
        container_cache: &mut HashMap<String, Container>,
    ) -> Result<(), DocumentError> {
        let tx = conn.transaction().map_err(DocumentError::DatabaseError)?;
        let count = documents.len();

        for mut scanned in documents {
            let document = &mut scanned.document;

            let container_id = if document.container_id.is_some() {
                document.container_id.unwrap()
            } else {
                let container_path = Path::new(&document.path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                container_cache.get(&container_path).unwrap().get_id()
            };

            tx.execute(
                "INSERT INTO documents (filename, extension, status, container_id) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![document.filename, document.extension, document.get_status_str(), container_id],
            )
            .map_err(|err| {
                if let rusqlite::Error::SqliteFailure(ref err_code, _) = err
                    && err_code.code == rusqlite::ErrorCode::ConstraintViolation{
                    return DocumentError::ConstraintViolation;
                }
                DocumentError::DatabaseError(err)
            })?;

            info!(target: LOG_TARGET, "Saved document: {}", document.path);

            let document_id = tx.last_insert_rowid();
            document.set_id(document_id);

            if document.content.is_empty() && document.description.is_empty() {
                continue;
            }

            tx.execute(
                "INSERT INTO index_documents (document_id, content, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![document_id, document.content, document.description],
            )
            .map_err(DocumentError::DatabaseError)?;
        }

        tx.commit().map_err(DocumentError::DatabaseError)?;

        info!(target: LOG_TARGET, "All documents saved successfully.");
        info!(target: LOG_TARGET, "Added documents: {}", count);

        Ok(())
    }

    pub fn get_format_type(&self) -> FormatType {
        FormatType::from_str(self.extension.as_deref().unwrap_or("")).unwrap_or(FormatType::Unknown)
    }

    pub fn get_status(&self) -> &DocumentStatus {
        &self.status
    }

    pub fn get_status_str(&self) -> &str {
        match self.status {
            DocumentStatus::New => "New",
            DocumentStatus::Scanned => "Scanned",
            DocumentStatus::Extracted => "Extracted",
            DocumentStatus::Classified => "Classified",
            DocumentStatus::Deleted => "Deleted",
        }
    }

    pub fn set_status(&mut self, status: DocumentStatus) {
        self.status = status;
    }
}
