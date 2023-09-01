use db::*;
use extractor::Extractor;
use handle::Handle;
use std::{error::Error, sync::mpsc, time::Duration};

pub mod db;
pub mod extractor;
pub mod graph_export;
pub mod handle;

#[macro_export]
macro_rules! make_extractors {
    ($($extractor:expr),*) => {
        {
            let mut extractors: Vec<Box<dyn $crate::extractor::Extractor>> = Vec::new();
            $(extractors.push(Box::new($extractor));)*
            extractors
        }
    };
}

pub fn run_extractors(
    write_log: mpsc::Receiver<(Entity, Attribute, Value)>,
    mut handle: &mut Handle,
    extractors: &mut Vec<Box<dyn Extractor>>,
) -> Result<(), Box<dyn Error>> {
    // Give all extractors a chance to initialize
    for extractor in extractors.iter_mut() {
        extractor.init(&mut handle)?;
    }

    // Run over all the stuff
    while let Ok((e, a, v)) = write_log.recv_timeout(Duration::from_millis(100)) {
        for extractor in extractors.iter_mut() {
            extractor.entry_added(&mut handle, &e, &a, &v).ok();
        }
    }

    Ok(())
}
