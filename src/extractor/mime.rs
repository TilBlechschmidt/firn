use super::Extractor;
use crate::{
    db::{Attribute, Entity, Value},
    handle::Handle,
};
use std::io::Read;

pub struct MimeInfer;

impl MimeInfer {
    pub fn attribute() -> Attribute {
        "type/mime".into()
    }
}

impl Extractor for MimeInfer {
    fn entry_added(
        &mut self,
        handle: &mut Handle,
        entity: &Entity,
        _attribute: &Attribute,
        _value: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mime_attribute = Self::attribute();

        // Ignore non-blobs
        if handle.get(entity, &"blob/size".into()).next().is_none() {
            return Ok(());
        }

        // Don't do the work twice :P
        if handle.get(entity, &mime_attribute).next().is_some() {
            return Ok(());
        }

        let mut info = infer::Infer::new();
        info.add("image/heic", "heic", custom_matcher::heic);

        // Fall back to reading the blob
        let mut blob = handle.blob(entity)?;
        let mut buf = [0; 32];
        blob.read(&mut buf)?;

        let mime = info
            .get(&buf)
            .map(|m| m.mime_type())
            .unwrap_or_else(|| "application/octet-stream".into());

        handle.insert(entity.clone(), mime_attribute, Value::Data(mime.into()));

        Ok(())
    }
}

mod custom_matcher {
    pub fn heic(buf: &[u8]) -> bool {
        const PATTERN: &[u8] = b"ftypheic";
        return buf.len() >= PATTERN.len() && &buf[4..4 + PATTERN.len()] == PATTERN;
    }
}
