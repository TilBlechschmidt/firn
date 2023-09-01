use firn::{db::*, extractor::*, handle::Handle, make_extractors, query, run_extractors};
use std::{
    env::current_dir,
    error::Error,
    fs::{copy, create_dir_all, File},
    path::PathBuf,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

fn main() -> Result<(), Box<dyn Error>> {
    let storage = current_dir()?.join("data/storage");
    std::fs::create_dir_all(&storage)?;

    let (database, write_log) = Database::new();
    let mut handle = Handle::new(database, storage.clone());

    let mut extractors = make_extractors![BlobLoader(storage.clone()), MimeInfer, ExifExtractor];

    run_extractors(write_log, &mut handle, &mut extractors)?;

    build_hierarchy(
        &mut handle,
        current_dir()?.join("data/hierarchy"),
        &storage,
        false,
    )?;

    Ok(())
}

fn build_hierarchy(
    handle: &mut Handle,
    root: impl Into<PathBuf>,
    storage: &PathBuf,
    fast_fake: bool,
) -> Result<(), Box<dyn Error>> {
    let root = root.into();
    std::fs::remove_dir_all(&root).ok();

    query!(handle where (?time, ?make, #model, #image) match [
        { #model, :"device/manufacturer", ?make },
        { #image, :"image/camera", #model },
        { #image, :"time/creation", ?time}
    ] => images);

    for entry in images {
        let make = entry.get(&make).unwrap().data();
        let time = entry.get(&time).unwrap().data();
        let model = &entry.get(&model).unwrap().0;
        let image = &entry.get(&image).unwrap().0;

        let datetime = OffsetDateTime::parse(time, &Rfc3339)?;
        let year = datetime.year();
        let month = datetime.month();
        let day = datetime.day();

        let path_cam = root.join("cam").join(format!("{make}/{model}"));
        let path_time = root.join("time").join(format!("{year}/{month}/{day}"));

        create_dir_all(&path_cam)?;
        create_dir_all(&path_time)?;

        if fast_fake {
            File::create(path_cam.join(image))?;
            File::create(path_time.join(image))?;
        } else {
            let src = storage.join(image);
            copy(&src, path_cam.join(image))?;
            copy(&src, path_time.join(image))?;
        }
    }

    Ok(())
}
