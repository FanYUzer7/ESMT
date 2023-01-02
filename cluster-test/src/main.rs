use MerkleRTree::shape::Rect;
use cluster_test::{ClusterArgs, read_dataset};
use structopt::StructOpt;

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
    let esmt = if &args.data_set == "imis" {
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
}
