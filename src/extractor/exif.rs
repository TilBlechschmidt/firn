use exif::{In, Tag};

use super::{Extractor, MimeInfer};
use crate::{
    attribute,
    db::{Attribute, Entity, Value},
    handle::Handle,
};

pub struct ExifExtractor;

impl ExifExtractor {
    fn extract_exif(
        &self,
        handle: &mut Handle,
        entity: &Entity,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut blob = handle.blob(entity)?;
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut blob)?;

        for f in exif.fields() {
            println!(
                "{} {} {}",
                f.tag,
                f.ifd_num,
                f.display_value().with_unit(&exif)
            );
        }

        if let Some(timestamp) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            // TODO Take Tag::OffsetOriginal into account for the time zone, yuck
            // TODO Convert to RFC3339 or whatever it was
            let timestamp = format!("{}", timestamp.display_value());
            handle.insert(
                entity.clone(),
                attribute!(time / creation),
                Value::Data(timestamp),
            );
        }

        if let Some(lat) = exif.get_field(Tag::GPSLatitude, In::PRIMARY) {
            println!("{}", lat.display_value());
        }

        Ok(())
    }
}

impl Extractor for ExifExtractor {
    fn entry_added(
        &mut self,
        handle: &mut Handle,
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if attribute == &MimeInfer::attribute() {
            match value {
                Value::Data(mime) if mime.starts_with("image") => {
                    self.extract_exif(handle, entity)?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
