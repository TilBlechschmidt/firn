use firn::{db::*, extractor::*, handle::Handle, make_extractors, run_extractors};
use std::{env::current_dir, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let storage = current_dir()?.join("data/storage");
    std::fs::create_dir_all(&storage)?;

    let (database, write_log) = Database::new();
    let mut handle = Handle::new(database, storage.clone());

    let mut extractors = make_extractors![
        BlobLoader(storage.clone()),
        MimeInfer,
        ExifExtractor // GeoNames::load(
                      //     "/Users/tibl/Downloads/DE/DE.txt",
                      //     "/Users/tibl/Downloads/hierarchy.txt"
                      // )?
    ];

    run_extractors(write_log, &mut handle, &mut extractors)?;

    println!("Created {} triplets", handle.len());

    Ok(())
}
