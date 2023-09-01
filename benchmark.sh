#!/bin/sh

sudo hyperfine --prepare "sync && purge" --min-runs 30 --export-json results.purge.json \
    ./target/release/lookup_single \
    ./target/release/lookup_enumeration \
    ./target/release/enumerate_flat \
    ./target/release/enumerate_tree \
    ./target/release/query_exif_flat \
    ./target/release/query_exif_tree \
    ./target/release/build_db

hyperfine --warmup 30 --min-runs 30 --export-json results.cache.json \
    ./target/release/lookup_single \
    ./target/release/lookup_enumeration \
    ./target/release/enumerate_flat \
    ./target/release/enumerate_tree \
    ./target/release/query_exif_flat \
    ./target/release/query_exif_tree \
    ./target/release/build_db

cargo bench --bench query -- --warm-up-time 10 --measurement-time 60

# python3 ~/Downloads/hyperfine/scripts/plot_whisker.py results.purge.json &
# python3 ~/Downloads/hyperfine/scripts/plot_whisker.py results.cache.json &
# open target/criterion/report/index.html
