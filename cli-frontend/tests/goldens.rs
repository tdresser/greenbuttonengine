use glob::glob;
use pretty_assertions::assert_eq;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn run_on_test_files(filetype: &str) -> String {
    let mut args: Vec<String> = vec![
        "run".into(),
        "--".into(),
        format!("--filetype={}", filetype).into(),
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
    println!("STDERR:\n{}", stderr);

    return stdout.to_owned();
}

#[test]
fn end_to_end_csv_golden() -> () {
    let golden = fs::read_to_string("./tests/goldens/golden.csv").unwrap();
    let stdout = run_on_test_files("csv");
    assert_eq!(golden, stdout);
}
