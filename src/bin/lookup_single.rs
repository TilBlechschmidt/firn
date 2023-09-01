use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let path: PathBuf =
        "data/hierarchy/time/2018/April/16/9A8A9515-4D61-4890-8B88-E251D2E94B52.heic".into();

    let metadata = std::fs::metadata(&path)?;

    let total_size = metadata.len();
    let total_name = path.file_name().unwrap().len();

    println!("Total size: {total_size}");
    println!("Total name: {total_name}");

    Ok(())
}
