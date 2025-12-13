use std::path::Path;

use tracing::info;
use tracing_subscriber::fmt::format::Format;

use crate::engine::extractor::formats::FormatType;

const LOG_TARGET: &str = "document";

#[derive(Debug)]
pub enum DocumentError {
    NotFound,
    ConstraintViolation,
    DatabaseError(rusqlite::Error),
}

#[derive(Debug, Clone)]
pub struct Document {
    id: Option<i64>,
    path: String,
    filename: String,
    extension: Option<String>,
    content: String,
    description: String,
}

impl Document {
    pub fn normalize_content(content: &str) -> String {
        content.to_ascii_uppercase()
    }

    pub fn new() -> Self {
        Document {
            id: None,
            path: String::new(),
            filename: String::new(),
            extension: None,
            content: String::new(),
            description: String::new(),
        }
    }

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
        self.content = Document::normalize_content(&content);
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

    pub fn save(&mut self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        conn.execute(
            "INSERT INTO documents (path, filename, extension) VALUES (?1, ?2, ?3)",
            rusqlite::params![self.path, self.filename, self.extension],
        )
        .map_err(|err| {
            if let rusqlite::Error::SqliteFailure(ref err_code, _) = err {
                if err_code.code == rusqlite::ErrorCode::ConstraintViolation {
                    return DocumentError::ConstraintViolation;
                }
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

    pub fn update_index(&self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        let document_id: i64 = self._get_id(conn)?;

        conn.execute(
            "UPDATE index_documents SET content = ?1, description = ?2 WHERE document_id = ?3",
            rusqlite::params![self.content, self.description, document_id],
        )
        .map_err(DocumentError::DatabaseError)?;

        Ok(())
    }

    pub fn update_metadata(&self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        conn.execute(
            "UPDATE documents SET path = ?1 WHERE path = ?2",
            rusqlite::params![self.path, self.path],
        )
        .map_err(DocumentError::DatabaseError)?;

        Ok(())
    }

    pub fn update(&self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        self.update_metadata(conn)?;
        self.update_index(conn)?;
        Ok(())
    }

    pub fn delete(&self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        let document_id: i64 = self._get_id(conn)?;

        conn.execute(
            "DELETE FROM index_documents WHERE document_id = ?1",
            rusqlite::params![document_id],
        )
        .map_err(DocumentError::DatabaseError)?;
        conn.execute(
            "DELETE FROM documents WHERE id = ?1",
            rusqlite::params![document_id],
        )
        .map_err(DocumentError::DatabaseError)?;
        Ok(())
    }

    pub fn get_id_by_path(&self, conn: &rusqlite::Connection) -> Result<i64, DocumentError> {
        let document_id: i64 = conn
            .query_row(
                "SELECT id FROM documents WHERE path = ?1",
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
            return Ok(id);
        } else {
            return self.get_id_by_path(conn);
        }
    }

    pub fn save_bulk(
        conn: &mut rusqlite::Connection,
        documents: Vec<Document>,
    ) -> Result<(), DocumentError> {
        let tx = conn.transaction().map_err(DocumentError::DatabaseError)?;
        let count = documents.len();

        for mut document in documents {
            tx.execute(
                "INSERT INTO documents (path, filename, extension) VALUES (?1, ?2, ?3)",
                rusqlite::params![document.path, document.filename, document.extension],
            )
            .map_err(|err| {
                if let rusqlite::Error::SqliteFailure(ref err_code, _) = err {
                    if err_code.code == rusqlite::ErrorCode::ConstraintViolation {
                        return DocumentError::ConstraintViolation;
                    }
                }
                DocumentError::DatabaseError(err)
            })?;

            let document_id = tx.last_insert_rowid();
            document.set_id(document_id);

            tx.execute(
                "INSERT INTO index_documents (document_id, content, description) VALUES (?1, ?2, ?3)",
                rusqlite::params![document_id, document.content, document.description],
            )
            .map_err(DocumentError::DatabaseError)?;

            info!(target: LOG_TARGET, "Saved document: {}", document.path);
        }

        tx.commit().map_err(DocumentError::DatabaseError)?;

        info!(target: LOG_TARGET, "All documents saved successfully.");
        info!(target: LOG_TARGET, "Total documents saved: {}", count);

        Ok(())
    }

    pub fn get_format_type(&self) -> FormatType {
        FormatType::get_by_extension(
            self.extension
                .as_deref()
                .unwrap_or(""),
        )
    }
}
