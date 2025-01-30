use glob::glob;
use pretty_assertions::assert_eq;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const TMP_OUTPUT_PATH: &str = "/tmp/golden_cmp";

fn run_on_test_files(filetype: &str) {
    let mut args: Vec<String> = vec![
        "run".into(),
        "--".into(),
        format!("--filetype={}", filetype).into(),
        format!("--out={}", TMP_OUTPUT_PATH).into(),
    ];

    let test_dir: PathBuf = [
        env::current_dir().unwrap().as_path(),
        Path::new("../test_files/*"),
    ]
    .iter()
    .collect();

    println!("{:?}", test_dir);
    let test_files = glob(test_dir.to_str().unwrap()).unwrap();
    let mut test_files: Vec<String> = test_files
        .into_iter()
        .map(|x| x.unwrap().to_str().unwrap().to_owned())
        .collect();
    args.append(&mut test_files);

    let output = Command::new("cargo")
        .args(args)
        .output()
        .expect("failed to execute process");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    if stderr.len() > 0 {
        println!("STDERR:\n{}", stderr);
    }
    if stdout.len() > 0 {
        println!("STDOUT:\n{}", stdout);
    }
}

#[test]
fn end_to_end_csv_golden() -> () {
    let golden = fs::read_to_string("./tests/goldens/golden.csv").unwrap();
    run_on_test_files("csv");
    let result = fs::read_to_string(TMP_OUTPUT_PATH).unwrap();
    assert_eq!(result, golden);
}

#[test]
fn end_to_end_influx_golden() -> () {
    let golden = fs::read_to_string("./tests/goldens/golden.influxdb").unwrap();
    run_on_test_files("influxdb");
    let result = fs::read_to_string(TMP_OUTPUT_PATH).unwrap();

    assert_eq!(golden, result);
}

#[test]
fn end_to_end_parquet_golden() -> () {
    let golden = fs::read("./tests/goldens/golden.parquet").unwrap();
    run_on_test_files("parquet");
    let result = fs::read(TMP_OUTPUT_PATH).unwrap();

    assert_eq!(golden, result);
}
