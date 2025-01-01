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

fn bench(c: &mut Criterion) {
    let files: Vec<_> = glob("../../test_files/*")
        .unwrap()
        .map(|path| {
            let path = &path.unwrap();
            return fs::read_to_string(path).expect("Should have been able to read the file");
        })
        .collect();
    c.bench_function("parse_test_files", |b| {
        b.iter(|| {
            println!("Start iteration.");
            run(&files);
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(30);
    targets = bench
);
criterion_main!(benches);
