#!/bin/bash

cargo run -- --filetype=csv ../../test_files/* --out=goldens/golden.csv
cargo run -- --filetype=influxdb ../../test_files/* --out=goldens/golden.influxdb
cargo run -- --filetype=parquet ../../test_files/* --out=goldens/golden.parquet