use firn::extractor::ExifExtractor;
use std::{error::Error, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let mut total_pixels = 0u128;

    for entry in std::fs::read_dir("data/storage")? {
        let file = entry?;

        if file.file_type()?.is_file() {
            let mut reader = BufReader::new(File::open(file.path())?);
            match ExifExtractor::extract(&mut reader) {
                Ok(exif) => {
                    total_pixels +=
                        (exif.width.unwrap_or_default() * exif.height.unwrap_or_default()) as u128;
                }
                Err(e) => {
                    println!("Failed to read {:?}: {e:?}", file.path());
                }
            }
        }
    }

    println!("Total pixels: {total_pixels}");

    Ok(())
}
