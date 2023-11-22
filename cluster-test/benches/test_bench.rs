use std::{path::PathBuf, str::FromStr};
use cluster_test::{read_dataset, utils::{MRTreeBuilder, ESMTreeBuilder}};
use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId};

pub const DATASET:&'static str = "uniform";
pub const SAMPLE_SIZE: usize = 20;

pub fn after_insert_test(c: &mut Criterion) {
    let data = if DATASET == "imis" {
        read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap()
    } else {
        read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap()
    };
    let mut group = c.benchmark_group("Structure Test");
    group.sample_size(SAMPLE_SIZE);
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000].iter() {
    // for i in [512000usize, 1024000].iter() {
        group.bench_with_input(BenchmarkId::new(format!("mrt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    MRTreeBuilder::new().base_size(*i).set_testset(&data).build_update_test(1.0)
                }, 
                |mrt| {
                    mrt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

criterion_group!(mybench, after_insert_test);
// criterion_group!(mybench, after_insert_test);
criterion_main!(mybench);