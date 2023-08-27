use db::{Attribute, Database, Entity, Value};
use extractor::*;
use handle::Handle;
use std::{
    env::current_dir,
    error::Error,
    time::{Duration, Instant},
};

mod db;
mod extractor;
mod handle;

struct ExtractorInstance {
    name: String,
    extractor: Box<dyn Extractor>,
    init_time: Duration,
    run_time: Duration,
}

impl ExtractorInstance {
    fn new(name: impl AsRef<str>, extractor: impl Extractor + 'static) -> Self {
        Self {
            name: name.as_ref().into(),
            extractor: Box::new(extractor),
            init_time: Duration::ZERO,
            run_time: Duration::ZERO,
        }
    }

    fn init(&mut self, handle: &mut Handle) {
        let start = Instant::now();

        if let Err(e) = self.extractor.init(handle) {
            println!("Extractor '{}' failed init: {e}", self.name);
        }

        self.init_time = start.elapsed();
    }

    fn entry_added(
        &mut self,
        handle: &mut Handle,
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) {
        let start = Instant::now();

        if let Err(e) = self.extractor.entry_added(handle, entity, attribute, value) {
            println!("Extractor '{}' failed: {e}", self.name);
        }

        self.run_time += start.elapsed();
    }
}

macro_rules! make_extractors {
    ($($name:expr => $extractor:expr),*) => {
        vec![
            $(ExtractorInstance::new($name, $extractor),)*
        ]
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let storage = current_dir()?.join("data/storage");
    std::fs::create_dir_all(&storage)?;

    let (database, write_log) = Database::new();
    let mut handle = Handle::new(database, storage.clone());

    let mut extractors = make_extractors![
        "loader" => BlobLoader(storage),
        "mime" => MimeInfer,
        "exif" => ExifExtractor,
        "log" => Logger
    ];

    // Give all extractors a chance to initialize
    for extractor in extractors.iter_mut() {
        extractor.init(&mut handle);
    }

    // Run over all the stuff
    while let Ok((e, a, v)) = write_log.recv_timeout(Duration::from_millis(100)) {
        for extractor in extractors.iter_mut() {
            extractor.entry_added(&mut handle, &e, &a, &v);
        }
    }

    // Print some stats
    for extractor in extractors {
        println!(
            "{:0>4}ms -> {:0>4}ms for {}",
            extractor.init_time.as_millis(),
            extractor.run_time.as_millis(),
            extractor.name
        );
    }

    Ok(())
}
