use std::error::Error;

use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    let mut total_size = 0;
    let mut total_name = 0;

    for entry in WalkDir::new("data/hierarchy") {
        let file = entry?;
        if file.file_type().is_file() {
            if let Ok(name) = file.file_name().to_owned().into_string() {
                total_size += file.metadata()?.len();
                total_name += name.len() as u64;
            }
        }
    }

    println!("Total size: {total_size}");
    println!("Total name: {total_name}");

    Ok(())
}
