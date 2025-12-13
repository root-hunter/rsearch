use crate::engine::utils;

#[derive(Debug)]
pub enum DocumentError {
    NotFound,
    ConstraintViolation,
    DatabaseError(rusqlite::Error),
}

#[derive(Debug)]
pub struct Document {
    path: String,
    filename: String,
    extension: String,
    content: String,
    description: String,
}

impl Document {
    pub fn new() -> Self {
        Document {
            path: String::new(),
            filename: String::new(),
            extension: String::new(),
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

    pub fn set_extension(&mut self, extension: String) {
        self.extension = extension;
    }

    pub fn get_extension(&self) -> &str {
        &self.extension
    }

    pub fn set_content(&mut self, content: String) {
        self.content = utils::normalize_content(&content);
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

    pub fn save(&self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
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

        Ok(())
    }

    pub fn update_index(&self, conn: &rusqlite::Connection) -> Result<(), DocumentError> {
        let document_id: i64 = self.get_id_by_path(conn)?;

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
        let document_id: i64 = self.get_id_by_path(conn)?;

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
}
