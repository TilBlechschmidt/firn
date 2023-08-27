use super::Extractor;
use crate::{
    attribute,
    db::{Attribute, Entity, Value},
    handle::Handle,
};
use std::io::Read;

pub struct MimeInfer;

impl MimeInfer {
    pub fn attribute() -> Attribute {
        attribute!(type / mime)
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

        // Don't do the work twice :P
        if handle.get(entity, &mime_attribute).next().is_some() {
            return Ok(());
        }

        // Fall back to reading the blob
        let mut blob = handle.blob(entity)?;
        let mut buf = [0; 8];
        blob.read(&mut buf)?;

        // TODO Sadly too stupid to detect HEIC files, find a better way
        //      Note that it would not even work based on the extension!
        let mime = tree_magic_mini::from_u8(&buf);
        handle.insert(entity.clone(), mime_attribute, Value::Data(mime.into()));

        Ok(())
    }
}
