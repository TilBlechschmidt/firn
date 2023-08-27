use super::db::Database;
use std::{
    fs::File,
    io::{self, Read, Seek},
    ops::Deref,
    path::PathBuf,
};

pub struct Handle {
    db: Database,
    storage: PathBuf,
}

impl Handle {
    pub fn new(db: Database, storage: PathBuf) -> Self {
        Self { db, storage }
    }

    pub fn pop_from_log(&mut self) -> sled::Result<Option<(String, String, String)>> {
        self.db.pop_from_log()
    }

    pub fn blob(&self, entity: &str) -> Result<impl Read + Seek, io::Error> {
        if entity.contains("/") {
            return Err(io::Error::new(io::ErrorKind::NotFound, "invalid entity ID"));
        }

        File::open(self.storage.join(entity))
    }
}

impl Deref for Handle {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
