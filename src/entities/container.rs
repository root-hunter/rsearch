use std::{collections::HashMap, path, str::FromStr};

use crate::{engine::scanner::ScannedDocument, entities::document::Document};

#[derive(Debug)]
pub enum ContainerError {
    DatabaseError(rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContainerType {
    Folder,
    Archive,
}

impl FromStr for ContainerType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Folder" => Ok(ContainerType::Folder),
            "Archive" => Ok(ContainerType::Archive),
            _ => Err(()),
        }
    }
}

impl ContainerType {
    pub fn to_str(&self) -> &str {
        match self {
            ContainerType::Folder => "Folder",
            ContainerType::Archive => "Archive",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Container {
    id: i64,
    path: String,
    r#type: ContainerType,
}

impl Container {
    pub fn new(id: i64, path: String, container_type: ContainerType) -> Self {
        Container {
            id,
            path,
            r#type: container_type,
        }
    }

    pub fn from_document(document: &Document, container_type: ContainerType) -> Self {
        let path = document.get_path();

        Container {
            id: 0,
            path: path.into(),
            r#type: container_type,
        }
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn update_cache_from_documents(
        conn: &mut rusqlite::Connection,
        documents: &[ScannedDocument],
        cache: &mut HashMap<String, Container>,
    ) -> Result<(), ContainerError> {
        for scanned in documents {
            let document = scanned.document.clone();
            let container_type = scanned.container_type.clone();

            let path = document.get_path();
            let path = path::Path::new(&path);
            if let Some(parent) = path.parent() {
                let container_path = parent.to_string_lossy().to_string();
                let container =
                    Container::get_or_create(conn, &container_path, container_type.clone(), cache)?;

                cache.insert(container_path, container);
            }
        }

        Ok(())
    }

    pub fn get_or_create(
        conn: &mut rusqlite::Connection,
        path: &str,
        container_type: ContainerType,
        cache: &mut HashMap<String, Container>,
    ) -> Result<Self, ContainerError> {
        if let Some(container) = cache.get(path) {
            return Ok(container.clone());
        }

        let mut stmt = conn
            .prepare(
                "INSERT INTO containers (path, type)
         VALUES (?1, ?2)
         ON CONFLICT(path) DO NOTHING
         RETURNING id, path, type",
            )
            .map_err(ContainerError::DatabaseError)?;

        let container_type_str = container_type.to_str();

        if let Ok(container) = stmt.query_row([path, container_type_str], |row| {
            let container_type_str: String = row.get(2)?;
            let container_type =
                ContainerType::from_str(&container_type_str).expect("Invalid container type");

            Ok(Container {
                id: row.get(0)?,
                path: row.get(1)?,
                r#type: container_type,
            })
        }) {
            return Ok(container);
        }

        conn.query_row(
            "SELECT id, path, type FROM containers WHERE path = ?1",
            [path],
            |row| {
                let container_type_str: String = row.get(2)?;
                let container_type =
                    ContainerType::from_str(&container_type_str).expect("Invalid container type");

                Ok(Container {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    r#type: container_type,
                })
            },
        )
        .map_err(ContainerError::DatabaseError)
    }

    pub fn save(&mut self, conn: &mut rusqlite::Connection) -> Result<(), ContainerError> {
        let mut stmt = conn
            .prepare(
                "INSERT INTO containers (path, type)
         VALUES (?1, ?2)
         ON CONFLICT(path) DO NOTHING
         RETURNING id",
            )
            .map_err(ContainerError::DatabaseError)?;

        let mut rows = stmt
            .query(rusqlite::params![self.path, self.r#type.to_str()])
            .map_err(ContainerError::DatabaseError)?;

        if let Some(row) = rows.next().map_err(ContainerError::DatabaseError)? {
            self.id = row
                .get::<_, i64>(0)
                .map_err(ContainerError::DatabaseError)?;
            return Ok(());
        }

        self.id = conn
            .query_row(
                "SELECT id FROM containers WHERE path = ?1",
                rusqlite::params![self.path],
                |row| row.get(0),
            )
            .map_err(ContainerError::DatabaseError)?;

        Ok(())
    }
}
