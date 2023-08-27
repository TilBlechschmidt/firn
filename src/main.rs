use db::Database;
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

    fn init(&mut self, handle: &Handle) {
        let start = Instant::now();

        if let Err(e) = self.extractor.init(handle) {
            println!("Extractor '{}' failed init: {e}", self.name);
        }

        self.init_time = start.elapsed();
    }

    fn entry_added(&mut self, handle: &Handle, entity: &str, attribute: &str, value: &str) {
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

    let mut handle = Handle::new(Database::temporary()?, storage.clone());

    let mut extractors = make_extractors![
        "loader" => BlobLoader(storage),
        "mime" => MimeInfer::new(),
        "log" => Logger
    ];

    // Give all extractors a chance to initialize
    for extractor in extractors.iter_mut() {
        extractor.init(&handle);
    }

    // Run over all the stuff
    while let Some((e, a, v)) = handle.pop_from_log()? {
        for extractor in extractors.iter_mut() {
            extractor.entry_added(&handle, &e, &a, &v);
        }
    }

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
