use std::{path::PathBuf, fs::File, io::{BufReader, BufRead}};
use std::str::FromStr;
use structopt::StructOpt;


#[derive(StructOpt, Debug)]
pub struct ClusterArgs {
    #[structopt(short = "t", long, default_value = "0")]
    pub types: u32,
    #[structopt(short = "d", long)]
    pub data_set: String,
    #[structopt(short = "f", long, parse(from_os_str))]
    pub file: PathBuf,
    #[structopt(short = "o", long, parse(from_os_str))]
    pub output: PathBuf,
}

pub fn read_dataset(data_set: &str, path: PathBuf) -> Result<Vec<[f64;2]>, String> {
    let file = File::open(path);
    if let Err(e) = file {
        return Err(format!("{:?}", e));
    }
    let file = file.unwrap();
    let data = match data_set {
        "dcw-p" => {
            Some(read_dcw_points(file))
        }
        "dcw-l" => {
            Some(read_dcw_lines(file))
        }
        "imis" => {
            Some(read_imis(file))
        }
        _ => { None }
    };

    data.ok_or("something error".to_string())
}

fn read_dcw_lines(file: File) -> Vec<[f64; 2]> {
    vec![]
}

fn read_dcw_points(file: File) -> Vec<[f64; 2]> {
    let mut data = vec![];
    let buffered = BufReader::new(file);
    for line in buffered.lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        let args = line
            .split(":")
            .map(|p| p.trim())
            .collect::<Vec<_>>();
        let point = [f64::from_str(args[0]).unwrap(), f64::from_str(args[1]).unwrap()];
        data.push(point);
    }
    data
}

fn read_imis(file: File) -> Vec<[f64; 2]> {
    vec![]
}