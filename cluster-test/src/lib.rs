use std::{path::PathBuf, fs::File, io::{BufReader, BufRead, BufWriter, Write}};
use std::str::FromStr;
use rand::{thread_rng, distributions::{Uniform}, prelude::Distribution};
use structopt::StructOpt;

pub mod utils;

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
    // println!("{:?}", std::env::current_exe());
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
        "uniform" => {
            Some(read_uniform(file))
        }
        _ => { None }
    };

    data.ok_or("something error".to_string())
}

fn read_dcw_lines(_file: File) -> Vec<[f64; 2]> {
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
    let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
    let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
    let mut data = vec![];
    let buffered = BufReader::new(file);
    for line in buffered.lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        let args = line
            .split(",")
            .collect::<Vec<_>>();
        let x = f64::from_str(args[0]).unwrap();
        let y = f64::from_str(args[1]).unwrap();
        if x > max_x {
            max_x = x;
        } else if x < min_x {
            min_x = x;
        }
        if y > max_y {
            max_y = y;
        } else if y < min_y {
            min_y = y;
        }
        let point = [x, y];
        data.push(point);
    }
    // min: [20.9999999936125, 35.0000449930892], max: [28.9999499908944, 38.9999999852576]
    // println!("data set range--> min: {:?}, max: {:?}", [min_x, min_y], [max_x, max_y]);
    data
}

fn read_uniform(file: File) -> Vec<[f64; 2]> {
    let mut data = vec![];
    let buffered = BufReader::new(file);
    for line in buffered.lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();
        let args = line
            .split(",")
            .collect::<Vec<_>>();
        let point = [f64::from_str(args[0]).unwrap(), f64::from_str(args[1]).unwrap()];
        data.push(point);
    }
    data
}

pub fn generate_uniform(min: [f64;2], max:[f64; 2], cnt: usize, path: PathBuf) -> Vec<[f64; 2]> {
    let file = File::create(path).unwrap();
    let mut buf = BufWriter::new(file);
    let mut rng = thread_rng();
    let x_sample = Uniform::new_inclusive(min[0], max[0]);
    let y_sample = Uniform::new_inclusive(min[1], max[1]);
    let data = (0..cnt).map(|_| [x_sample.sample(&mut rng), y_sample.sample(&mut rng)]).collect::<Vec<_>>();
    for &p in &data {
        let _ =buf.write(format!("{},{}\n", p[0],p[1]).as_bytes());
    }
    data
}