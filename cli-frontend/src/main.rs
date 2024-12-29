use std::fs;

use anyhow::anyhow;
use anyhow::Result;
use clap::{Parser, ValueEnum};
use personalgreenbutton::{parse_xml, TimeSeries};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FileType {
    CSV,
    Influxdb,
    Parquet,
}

#[derive(Parser)]
struct Cli {
    /// Input files.
    #[arg(short, long, value_enum, value_name = "FILETYPE")]
    filetype: FileType,
    /// Output file (optional, except for parquet).
    #[arg(short, long)]
    out: Option<std::path::PathBuf>,
    /// Paths of input files.
    paths: Vec<std::path::PathBuf>,
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut timeseries = TimeSeries::default();
    for path in &cli.paths {
        let xml = fs::read_to_string(path).expect("Should have been able to read the file");
        let result = parse_xml(&xml);
        match result {
            Ok(x) => timeseries.extend(x),
            Err(x) => eprintln!("Failed to read file {} {}", path.to_str().unwrap(), x),
        }
    }

    let mut str_out: Option<String> = None;
    match cli.filetype {
        FileType::CSV => str_out = Some(timeseries.as_csv().map_err(|x| anyhow!(x))?),
        FileType::Influxdb => str_out = Some(timeseries.as_influxdb()),
        FileType::Parquet => {
            let buf = timeseries.as_parquet().map_err(|x| anyhow!(x))?;
            match &cli.out {
                Some(path) => std::fs::write(path, &buf).unwrap(),
                None => panic!("outfile required with parquet."),
            };
        }
    }
    if let Some(str_out) = str_out {
        match &cli.out {
            Some(path) => std::fs::write(path, &str_out).unwrap(),
            None => println!("{}", str_out),
        }
    }
    return Ok(());
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(x) => panic!("{:?}", x),
    }
}
