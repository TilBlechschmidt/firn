use super::Extractor;
use crate::{db::Value, handle::Handle};
use std::path::PathBuf;

pub struct BlobLoader(pub PathBuf);

impl Extractor for BlobLoader {
    fn init(&mut self, handle: &mut Handle) -> Result<(), Box<dyn std::error::Error>> {
        let mut i = 0;
        for entry in std::fs::read_dir(&self.0)? {
            let file = entry?;
            if file.file_type()?.is_file() {
                if let Ok(name) = file.file_name().into_string() {
                    let size = file.metadata()?.len();
                    handle.insert(name, "blob/size", Value::Data(size.to_string()));
                    i += 1;
                }
            }

            // if i > 200 {
            //     break;
            // }
        }

        Ok(())
    }
}
