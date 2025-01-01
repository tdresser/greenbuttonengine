use criterion::{criterion_group, criterion_main, Criterion};
use glob::glob;
use personalgreenbutton::{parse_xml, TimeSeries};
use std::fs;

fn run(files: &[String]) -> TimeSeries {
    let mut timeseries = TimeSeries::default();
    for xml in files {
        let result = parse_xml(&xml);
        match result {
            Ok(x) => timeseries.extend(x),
            Err(x) => panic!("Parse failure: {:?}", x),
        }
    }
    return timeseries;
}

fn get_test_files() -> Vec<String> {
    return glob("../../test_files/*")
        .unwrap()
        .map(|path| {
            let path = &path.unwrap();
            return fs::read_to_string(path).expect("Should have been able to read the file");
        })
        .collect();
}

fn parse(c: &mut Criterion) {
    let files = get_test_files();
    c.bench_function("parse_test_files", |b| {
        b.iter(|| {
            run(&files);
        })
    });
}

fn sort(c: &mut Criterion) {
    let timeseries = run(&get_test_files());
    c.bench_function("sort", |b| {
        b.iter(|| {
            timeseries.clone().sort_and_chunk();
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(30);
    targets = parse, sort
);
criterion_main!(benches);
