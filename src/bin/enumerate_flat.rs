use std::{env::current_dir, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let storage = current_dir()?.join("data/storage");

    let mut total_size = 0;
    let mut total_name = 0;

    for entry in std::fs::read_dir(&storage)? {
        let file = entry?;
        if file.file_type()?.is_file() {
            if let Ok(name) = file.file_name().into_string() {
                total_size += file.metadata()?.len();
                total_name += name.len() as u64;
            }
        }
    }

    println!("Total size: {total_size}");
    println!("Total name: {total_name}");

    Ok(())
}
