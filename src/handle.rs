use super::db::{Database, Entity};
use std::{
    fs::File,
    io::{self, BufRead, Read, Seek},
    ops::{Deref, DerefMut},
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

    pub fn blob(&self, entity: &Entity) -> Result<impl BufRead + Read + Seek, io::Error> {
        if entity.0.contains("/") {
            return Err(io::Error::new(io::ErrorKind::NotFound, "invalid entity ID"));
        }

        let file = File::open(self.storage.join(&entity.0))?;
        Ok(std::io::BufReader::new(file))
    }
}

impl Deref for Handle {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for Handle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}
