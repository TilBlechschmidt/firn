use super::Extractor;
use sled::{Config, Db, Mode};
use std::{env::current_dir, fs::create_dir_all, io::Read};

const MIME_ATTRIBUTE: &str = "type/mime";

pub struct MimeInfer {
    cache: Db,
}

impl MimeInfer {
    pub fn new() -> Self {
        let path = current_dir()
            .expect("failed to get current dir")
            .join("data/cache/mime");

        create_dir_all(&path).expect("failed to create mime cache dir");

        let cache = Config::new()
            .use_compression(false)
            .mode(Mode::HighThroughput)
            .path(path)
            .open()
            .expect("failed to open mime cache database");

        Self { cache }
    }

    fn read_cache(&self, entity: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self.cache.get(entity)? {
            Some(data) => Ok(Some(String::from_utf8(data.to_vec())?)),
            None => Ok(None),
        }
    }
}

impl Extractor for MimeInfer {
    fn entry_added(
        &mut self,
        handle: &crate::handle::Handle,
        entity: &str,
        _attribute: &str,
        _value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Don't do the work twice :P
        if handle.get(entity, MIME_ATTRIBUTE).next().is_some() {
            return Ok(());
        }

        // Use cache if possible
        match self.read_cache(entity) {
            Ok(Some(mime)) => {
                handle.insert(entity, MIME_ATTRIBUTE, mime)?;
                return Ok(());
            }
            Err(e) => eprintln!("failed to read mime cache: {e}"),
            _ => {}
        }

        // Fall back to reading the blob
        let mut blob = handle.blob(entity)?;
        let mut buf = [0; 8];
        blob.read(&mut buf)?;

        if let Some(matched) = infer::get(&buf) {
            let mime = matched.mime_type();
            self.cache.insert(entity, mime)?;
            handle.insert(entity, MIME_ATTRIBUTE, mime)?;
        } else {
            println!("FAILED TO FIND MIME");
        }

        Ok(())
    }
}
