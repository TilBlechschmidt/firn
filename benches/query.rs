use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use firn::{db::*, extractor::*, handle::Handle, make_extractors, query, run_extractors};
use std::{env::current_dir, error::Error};

fn build_db() -> Result<Handle, Box<dyn Error>> {
    let storage = current_dir()?.join("data/storage");
    std::fs::create_dir_all(&storage)?;

    let (database, write_log) = Database::new();
    let mut handle = Handle::new(database, storage.clone());

    let mut extractors = make_extractors![
        BlobLoader(storage.clone()),
        MimeInfer,
        ExifExtractor,
        GeoNames::load(
            "/Users/tibl/Downloads/DE/DE.txt",
            "/Users/tibl/Downloads/hierarchy.txt"
        )?
    ];

    run_extractors(write_log, &mut handle, &mut extractors)?;

    Ok(handle)
}

fn query_all(handle: &Handle) -> usize {
    query!(handle where (#e, :a, ?v) match [
        { #e, :a, ?v }
    ] => triplets);

    triplets.len()
}

fn query_blobs(handle: &Handle) -> usize {
    query!(handle where (#e, ?v) match [
        { #e, :"blob/size", ?v }
    ] => triplets);

    triplets.len()
}

fn query_location(handle: &Handle) -> usize {
    // TODO Runs into an infinite loop ... find out why
    // query!(handle where (#sh, #region, #place, #image) match [
    //     { #sh, :"text/label", ?"Schleswig-Holstein" },
    //     { #region, :"relation/parent", #sh },
    //     { #place, :"relation/parent", #region },
    //     { #image, :"location/geoname", #place }
    // ] => images);

    query!(handle where (#town, #image) match [
        { #town, :"text/label", ?"Geesthacht" },
        { #image, :"location/geoname", #town }
    ] => images);

    images.len()
}

fn criterion_benchmark(c: &mut Criterion) {
    let handle = build_db().unwrap();

    c.bench_with_input(
        BenchmarkId::new("query_all", handle.len()),
        &handle,
        |b, h| b.iter(|| query_all(h)),
    );

    c.bench_with_input(
        BenchmarkId::new("query_blobs", handle.len()),
        &handle,
        |b, h| b.iter(|| query_blobs(h)),
    );

    c.bench_with_input(
        BenchmarkId::new("query_location", handle.len()),
        &handle,
        |b, h| b.iter(|| query_location(h)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
