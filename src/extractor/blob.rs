use super::Extractor;
use crate::{
    attribute,
    db::{Attribute, Value},
    handle::Handle,
};
use std::path::PathBuf;

pub struct BlobLoader(pub PathBuf);

impl Extractor for BlobLoader {
    fn init(&mut self, handle: &mut Handle) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(&self.0)? {
            let file = entry?;
            if file.file_type()?.is_file() {
                if let Ok(name) = file.file_name().into_string() {
                    let size = file.metadata()?.len();
                    handle.insert(name, attribute!(blob / size), Value::Data(size.to_string()));
                }
            }
        }

        Ok(())
    }
}
