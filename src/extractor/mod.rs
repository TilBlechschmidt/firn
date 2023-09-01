use crate::{
    db::{Attribute, Entity, Value},
    handle::Handle,
};
use std::error::Error;

mod blob;
mod exif;
mod geonames;
mod mime;

pub use blob::BlobLoader;
pub use exif::ExifExtractor;
pub use geonames::GeoNames;
pub use mime::MimeInfer;

pub trait Extractor {
    #[allow(unused_variables)]
    fn init(&mut self, handle: &mut Handle) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn entry_added(
        &mut self,
        handle: &mut Handle,
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub struct Logger;

impl Extractor for Logger {
    fn entry_added(
        &mut self,
        _handle: &mut Handle,
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> Result<(), Box<dyn Error>> {
        println!("+ {entity:?} :{attribute:?} {value:?}");
        Ok(())
    }
}
