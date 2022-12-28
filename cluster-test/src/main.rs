use std::{time::{Instant, Duration}, ops::AddAssign, path::PathBuf, str::FromStr};

use MerkleRTree::shape::Rect;
use cluster_test::{ClusterArgs, read_dataset};
use structopt::StructOpt;
use types::hash_value::HashValue;

fn main() {
    let args = ClusterArgs::from_args();
    println!("{:?}", args.file);
    let data = read_dataset(&args.data_set, args.file.clone());
    // let data_set = "dcw-p";
    // let mut file = PathBuf::from_str("../release/data_set/dcw-points/NApppoint.fnl").unwrap();
    // println!("{:?}", file);
    // let data = read_dataset(data_set, file);
    if let Err(e) = data {
        println!("{}", e);
        return;
    }
    
    let data = data.unwrap();
    let mut mrt = MerkleRTree::mrtree::MerkleRTree::<f64, 2, 4>::new();
    let mut esmt = MerkleRTree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([-180.0f64, 14.5f64], [-50.0f64, 80.0f64]), 3);
    let mut dur_mrt = Duration::new(0,0);
    let mut dur_esmt = Duration::new(0,0);
    let mut avg_mrt = vec![];
    let mut avg_esmt = vec![];
    for (idx, point) in data.into_iter().enumerate() {
        println!("insert {}th point", idx);
        let p_clone = point.clone();
        let key_mrt = format!("keymrt-{}", idx);
        let key_esmt = format!("keyesmt-{}", idx);
        let hash_mrt = HashValue::default();
        let hash_esmt = HashValue::default();

        let ins_mrt = Instant::now();
        mrt.insert(key_mrt, p_clone, hash_mrt);
        dur_mrt.add_assign(ins_mrt.elapsed());

        let ins_esmt = Instant::now();
        esmt.insert(key_esmt, point, hash_esmt);
        dur_esmt.add_assign(ins_esmt.elapsed());

        avg_mrt.push(dur_mrt.as_nanos() / (idx + 1) as u128);
        avg_esmt.push(dur_esmt.as_nanos() / (idx + 1) as u128);
    }
    for idx in [99usize, 199, 399, 799, 1599, 3199, 6399, 12799, 25599] {
        if idx >= avg_esmt.len() {
            break;
        }
        println!("size = {}, avg insert time--> mrt: {}ns, esmt: {}ns", idx + 1, avg_mrt[idx], avg_esmt[idx]);
    }
}
