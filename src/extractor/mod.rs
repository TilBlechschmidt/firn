use crate::handle::Handle;
use std::error::Error;

mod blob;
mod mime;

pub use blob::BlobLoader;
pub use mime::MimeInfer;

pub trait Extractor {
    #[allow(unused_variables)]
    fn init(&mut self, handle: &Handle) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn entry_added(
        &mut self,
        handle: &Handle,
        entity: &str,
        attribute: &str,
        value: &str,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

pub struct Logger;

impl Extractor for Logger {
    fn entry_added(
        &mut self,
        _handle: &Handle,
        entity: &str,
        attribute: &str,
        value: &str,
    ) -> Result<(), Box<dyn Error>> {
        println!("+ E: {entity} | A: {attribute} | V: {value}");
        Ok(())
    }
}
