use db::{Attribute, Database, Entity, Rule, Value, Variable, VariableSetExt};
use extractor::*;
use handle::Handle;
use std::{
    env::current_dir,
    error::Error,
    fs::{create_dir_all, File},
    path::PathBuf,
    time::{Duration, Instant},
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

mod db;
mod extractor;
mod graph_export;
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
        "geonames" => GeoNames::load("/Users/tibl/Downloads/DE/DE.txt", "/Users/tibl/Downloads/hierarchy.txt")?
        // "log" => Logger
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

    println!("{} triplets stored", handle.len());

    // query!(handle where (?time, ?make, #model, #image) match [
    //     { #model, :"device/manufacturer", ?make },
    //     { #image, :"image/camera", #model },
    //     { #image, :"time/creation", ?time}
    // ] => images);

    // print_results!(images => make, model, image, time);

    // query!(handle where (#a, #b, ?label) match [
    //     { #a, :"relation/parent", #b },
    //     { #a, :"text/label", ?label }
    // ] => geonames);

    // print_results!(geonames => a, label);

    query!(handle where (#town, #image) match [
        { #town, :"text/label", ?"Geesthacht" },
        { #image, :"location/geoname", #town }
    ] => images);

    dbg!(images.len());

    // let start = Instant::now();

    // query!(handle where (#camera, ?make) match [
    //     { #camera, :"device/manufacturer", ?make}
    // ] => cameras);

    // query!(handle where (#image) match [
    //     { #image, :"image/camera", #Entity::from("Rollei Bullet 3S 720P") }
    // ] => images);

    // println!("{} ns", start.elapsed().as_nanos());

    // print_results!(cameras => camera, make);
    // print_results!(images => image);

    // std::thread::sleep(Duration::from_secs(15));

    // graph_export::export_graph(&handle)?;

    Ok(())
}
