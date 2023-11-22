use std::{path::PathBuf, str::FromStr, time::Instant, os::unix::thread};

use authentic_rtree::shape::Rect;
use cluster_test::{ClusterArgs, read_dataset, utils::{manu_test, MRTreeBuilder, ESMTreeBuilder}};
use structopt::StructOpt;

fn main() {
    // let args = ClusterArgs::from_args();
    // println!("{:?}", args.file);
    // let data = read_dataset(&args.data_set, args.file.clone());
    // if let Err(e) = data {
    //     println!("{}", e);
    //     return;
    // }
    
    // let data = data.unwrap();
    // println!("data length = {}", data.len());
    // // let mut mrt = authentic_rtree::mrtree::authentic_rtree::<f64, 2, 4>::new();
    // // let mut esmt = authentic_rtree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([-180.0f64, 14.5f64], [-50.0f64, 80.0f64]), 4);
    // let esmt = if &args.data_set == "imis" {
    //     authentic_rtree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576]), 4)
    // } else {
    //     authentic_rtree::esmtree::PartionManager::<f64, 2, 4>::new(Rect::new([0.0, 0.0], [160.0, 160.0]), 4)
    // };
    // {
    //     let mut p = Vec::from([0; 256]);
    //     let mut d = vec![0;256];
    //     for d in &data {
    //         let idx = esmt.point_index(d) - 85;
    //         p[idx] += 1;
    //     }
    //     let mut idx = 0usize;
    //     for l1 in 0..4usize {
    //         let x1 = l1 >> 1;
    //         let y1 = l1 & 1;
    //         for l2 in 0..4usize {
    //             let x2 = l2 >> 1;
    //             let y2 = l2 & 1;
    //             for l3 in 0..4usize {
    //                 let x3 = l3 >> 1;
    //                 let y3 = l3 & 1;
    //                 for l4 in 0..4usize {
    //                     let x4 = l4 >> 1;
    //                     let y4 = l4 & 1;
    //                     let x = (x1 << 3) + (x2 << 2) + (x3 << 1) + x4;
    //                     let y = (y1 << 3) + (y2 << 2) + (y3 << 1) + y4;
    //                     d[y*16 + x] = idx;
    //                     idx += 1;
    //                 }
    //             }
    //         }
    //     }
    //     print!("[");
    //     for i in 0..16usize {
    //         print!("[{}", p[d[i*16]]);
    //         for j in 1..16usize {
    //             print!(",{}", p[d[i*16+j]]);
    //         }
    //         println!("],");
    //     }
    //     println!("]");
    // }
    // 
    let uni_data = read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap();
    // let test_set = ["construct", "insert", "delete", "update", "query", "batch_construct", "batch_insert", "edge", "parallel", "query_after_batch"];
    let test_set = ["query_after_batch"];
    let data_size = [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000];
    let query_size = 0.1f64;
    let traverse_size = 0.8f64;
    let mut mrt_value = 0.0f64;
    let mut esmt_value = 0.0f64;
    for test in test_set {
        for i in data_size.iter() {
            let test_timer = Instant::now();
            println!("TEST {}/{} start", test, i);
            mrt_value = 0.0f64;
            esmt_value = 0.0f64;
            let mut thread_time = vec![0.0f64;16];
            for _ in 0..10 {
                match test {
                    "construct" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&uni_data).build_insert_test();
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_insert_test();
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1;
                    }
                    "insert" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&uni_data).build_update_test(1.0);
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_update_test(1.0);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "delete" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&uni_data).build_delete_test();
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_delete_test();
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "update" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&uni_data).build_update_test(0.0);
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_update_test(0.0);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "query" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&uni_data).build_query_test();
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_query_test(query_size);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "batch_construct" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_batch_construct(1000);
                        let o_esmt = esmt.exec_batch();
                        esmt_value += o_esmt.1;
                    }
                    "batch_insert" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i).opt_size(1000)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_batch_insert(1000);
                        let o_esmt = esmt.exec_batch();
                        esmt_value += o_esmt.1;
                    }
                    "edge" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_query_test(traverse_size);
                        let o_esmt = esmt.exec();
                        mrt_value += o_esmt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_edge_test(traverse_size);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "parallel" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_query_test(traverse_size);
                        let o_esmt = esmt.exec();
                        mrt_value += o_esmt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_query_test(traverse_size);
                        let o_esmt = esmt.exec_parallel();
                        esmt_value += o_esmt.1*10.0f64;
                        for i in 0..16usize {
                            thread_time[i] += o_esmt.2[i]/1000.0f64;
                        }
                    }
                    "query_after_batch" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_query_test(query_size);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).batch_build_iter_query(1000, query_size);
                        let o_esmt = esmt.exec();
                        mrt_value += o_esmt.1*10.0f64;
                    }
                    _ => {
                        
                    }
                }
            }
            println!("\t{}/{}/mrt\tavg time: {:.4}\tms", test, i, mrt_value/10.0f64);
            println!("\t{}/{}/esmt\tavg time: {:.4}\tms", test, i, esmt_value/10.0f64);
            if mrt_value > 0.0f64 {
                println!("\t{}/{}\t\timproved: {:.4}%", test, i, (mrt_value-esmt_value)/mrt_value);
            }
            println!("TEST {}/{} finished.\ttime comsume: {} s", test, i, Instant::elapsed(&test_timer).as_secs_f64());
            println!("thread time: {:?}", thread_time);
            println!("------------------------");
            println!();
        }
    }

    println!("================================================================");
    println!("Now test imis");
    let imis_data = read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap();

    for test in test_set {
        for i in data_size.iter() {
            let test_timer = Instant::now();
            println!("TEST {}/{} start", test, i);
            mrt_value = 0.0f64;
            esmt_value = 0.0f64;
            for _ in 0..10 {
                match test {
                    "construct" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&imis_data).build_insert_test();
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_insert_test();
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1;
                    }
                    "insert" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&imis_data).build_update_test(1.0);
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_update_test(1.0);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "delete" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&imis_data).build_delete_test();
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_delete_test();
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "update" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&imis_data).build_update_test(0.0);
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_update_test(0.0);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "query" => {
                        let mrt = MRTreeBuilder::new().base_size(*i).set_testset(&imis_data).build_query_test();
                        let o_mrt = mrt.exec();
                        mrt_value += o_mrt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_query_test(query_size);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    "batch_construct" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_batch_construct(1000);
                        let o_esmt = esmt.exec_batch();
                        esmt_value += o_esmt.1;
                    }
                    "batch_insert" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i).opt_size(1000)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&imis_data).build_batch_insert(1000);
                        let o_esmt = esmt.exec_batch();
                        esmt_value += o_esmt.1;
                    }
                    "edge" => {
                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_query_test(traverse_size);
                        let o_esmt = esmt.exec();
                        mrt_value += o_esmt.1*10.0f64;

                        let esmt = ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&uni_data).build_edge_test(traverse_size);
                        let o_esmt = esmt.exec();
                        esmt_value += o_esmt.1*10.0f64;
                    }
                    _ => {
                        
                    }
                }
            }
            println!("\t{}/{}/mrt\tavg time: {:.4}\tms", test, i, mrt_value/10.0f64);
            println!("\t{}/{}/esmt\tavg time: {:.4}\tms", test, i, esmt_value/10.0f64);
            if mrt_value > 0.0f64 {
                println!("\t{}/{}\t\timproved: {:.4}%", test, i, (mrt_value-esmt_value)/mrt_value);
            }
            println!("TEST {}/{} finished.\ttime comsume: {} s", test, i, Instant::elapsed(&test_timer).as_secs_f64());
            println!("------------------------");
            println!();
        }
    }
}
