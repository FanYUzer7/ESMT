use std::{time::{Instant, Duration}, ops::AddAssign};

use MerkleRTree::shape::Rect;
use cluster_test::{ClusterArgs, read_dataset, generate_uniform};
use rand::{thread_rng, Rng};
use structopt::StructOpt;
use types::hash_value::HashValue;

fn main() {
    let args = ClusterArgs::from_args();
    println!("{:?}", args.file);
    let data = read_dataset(&args.data_set, args.file.clone());
    if let Err(e) = data {
        println!("{}", e);
        return;
    }
    
    let data = data.unwrap();
    println!("data length = {}", data.len());
    // let mut mrt = MerkleRTree::mrtree::MerkleRTree::<f64, 2, 4>::new();
    // let mut esmt = MerkleRTree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([-180.0f64, 14.5f64], [-50.0f64, 80.0f64]), 4);
    let mut esmt = if &args.data_set == "imis" {
        MerkleRTree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576]), 4)
    } else {
        MerkleRTree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([0.0, 0.0], [160.0, 160.0]), 4)
    };
    {
        let mut p = Vec::from([0; 256]);
        let mut d = vec![0;256];
        for d in &data {
            let idx = esmt.point_index(d) - 85;
            p[idx] += 1;
        }
        let mut idx = 0usize;
        for l1 in 0..4usize {
            let x1 = l1 >> 1;
            let y1 = l1 & 1;
            for l2 in 0..4usize {
                let x2 = l2 >> 1;
                let y2 = l2 & 1;
                for l3 in 0..4usize {
                    let x3 = l3 >> 1;
                    let y3 = l3 & 1;
                    for l4 in 0..4usize {
                        let x4 = l4 >> 1;
                        let y4 = l4 & 1;
                        let x = (x1 << 3) + (x2 << 2) + (x3 << 1) + x4;
                        let y = (y1 << 3) + (y2 << 2) + (y3 << 1) + y4;
                        d[y*16 + x] = idx;
                        idx += 1;
                    }
                }
            }
        }
        print!("[");
        for i in 0..16usize {
            print!("[{}", p[d[i*16]]);
            for j in 1..16usize {
                print!(",{}", p[d[i*16+j]]);
            }
            println!("],");
        }
        println!("]");
    }
    // let mut dur_mrt = Duration::new(0,0);
    // let mut dur_esmt = Duration::new(0,0);
    // let mut avg_mrt = vec![];
    // let mut avg_esmt = vec![];
    // for (idx, point) in data.into_iter().enumerate() {
    //     println!("insert {}th point", idx);
    //     let p_clone = point.clone();
    //     let key_mrt = format!("keymrt-{}", idx);
    //     let key_esmt = format!("keyesmt-{}", idx);
    //     let hash_mrt = HashValue::default();
    //     let hash_esmt = HashValue::default();

    //     let ins_mrt = Instant::now();
    //     mrt.insert(key_mrt, p_clone, hash_mrt);
    //     dur_mrt.add_assign(ins_mrt.elapsed());

    //     let ins_esmt = Instant::now();
    //     esmt.insert(key_esmt, point, hash_esmt);
    //     dur_esmt.add_assign(ins_esmt.elapsed());

    //     avg_mrt.push(dur_mrt.as_nanos() / (idx + 1) as u128);
    //     avg_esmt.push(dur_esmt.as_nanos() / (idx + 1) as u128);
    // }
    // for idx in [99usize, 199, 399, 799, 1599, 3199, 6399, 12799, 25599] {
    //     if idx >= avg_esmt.len() {
    //         break;
    //     }
    //     println!("size = {}, avg insert time--> mrt: {}ns, esmt: {}ns", idx + 1, avg_mrt[idx], avg_esmt[idx]);
    // }

    // let mut query = vec![];
    // let mut rng = thread_rng();
    // for _ in 0..1000 {
    //     let x = rng.gen_range(-167.0f64..=-63.0f64);
    //     let y = rng.gen_range(21.05f64..=73.4f64);
    //     query.push(Rect::new([x-6.5, y-3.275], [x+6.5, y+3.275]));
    // }
    // dur_mrt = Duration::new(0,0);
    // dur_esmt = Duration::new(0,0);
    // for q in &query {
    //     let ins_mrt = Instant::now();
    //     let _ = mrt.range_query(q);
    //     dur_mrt.add_assign(ins_mrt.elapsed());

    //     let ins_esmt = Instant::now();
    //     esmt.range_query(q);
    //     dur_esmt.add_assign(ins_esmt.elapsed());
    // }
    // println!("avg query time--> mrt: {}ns, esmt: {}ns", dur_mrt.as_nanos() / 1000, dur_esmt.as_nanos() / 1000);
}
