use std::{path::PathBuf, str::FromStr};
use cluster_test::{read_dataset, utils::{MRTreeBuilder, ESMTreeBuilder}};
use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId};

pub const DATASET:&'static str = "imis";
pub const SAMPLE_SIZE: usize = 20;

pub fn continuous_insert_test(c: &mut Criterion) {
    let data = if DATASET == "imis" {
        read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap()
    } else {
        read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap()
    };
    let mut group = c.benchmark_group("Continuous-Insert");
    group.sample_size(SAMPLE_SIZE);
    // group.plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [1000usize, 2000, 4000].iter() {
        group.bench_with_input(BenchmarkId::new(format!("mrt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    MRTreeBuilder::new().base_size(*i).set_testset(&data).build_insert_test()
                }, 
                |mrt| {
                    mrt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
        group.bench_with_input(BenchmarkId::new(format!("esmt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    if DATASET == "imis" {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&data).build_insert_test()
                    } else {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_insert_test()
                    }
                }, 
                |esmt| {
                    esmt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

pub fn after_insert_test(c: &mut Criterion) {
    let data = if DATASET == "imis" {
        read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap()
    } else {
        read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap()
    };
    let mut group = c.benchmark_group("After-Insert");
    group.sample_size(SAMPLE_SIZE);
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [1000usize, 2000, 4000].iter() {
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
        group.bench_with_input(BenchmarkId::new(format!("esmt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    if DATASET == "imis" {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&data).build_update_test(1.0)
                    } else {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_update_test(1.0)
                    }
                }, 
                |esmt| {
                    esmt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

pub fn delete_test(c: &mut Criterion) {
    let data = if DATASET == "imis" {
        read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap()
    } else {
        read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap()
    };
    let mut group = c.benchmark_group("Delete");
    group.sample_size(SAMPLE_SIZE);
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [1000usize, 2000, 4000].iter() {
        group.bench_with_input(BenchmarkId::new(format!("mrt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    MRTreeBuilder::new().base_size(*i).set_testset(&data).build_delete_test()
                }, 
                |mrt| {
                    mrt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
        group.bench_with_input(BenchmarkId::new(format!("esmt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    if DATASET == "imis" {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&data).build_delete_test()
                    } else {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_delete_test()
                    }
                }, 
                |esmt| {
                    esmt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

pub fn update_full_test(c: &mut Criterion) {
    let data = if DATASET == "imis" {
        read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap()
    } else {
        read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap()
    };
    let mut group = c.benchmark_group("Update-full");
    group.sample_size(SAMPLE_SIZE);
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [256000usize].iter() {
        group.bench_with_input(BenchmarkId::new(format!("mrt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    MRTreeBuilder::new().base_size(*i).set_testset(&data).build_update_test(0.0)
                }, 
                |mrt| {
                    mrt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
        group.bench_with_input(BenchmarkId::new(format!("esmt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    if DATASET == "imis" {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&data).build_update_test(0.0)
                    } else {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_update_test(0.0)
                    }
                }, 
                |esmt| {
                    esmt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

pub fn range_query_test(c: &mut Criterion) {
    let data = if DATASET == "imis" {
        read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap()
    } else {
        read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap()
    };
    let mut group = c.benchmark_group("Range-query");
    group.sample_size(SAMPLE_SIZE);
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [1000usize, 2000, 4000].iter() {
        group.bench_with_input(BenchmarkId::new(format!("mrt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    MRTreeBuilder::new().base_size(*i).set_testset(&data).build_query_test()
                }, 
                |mrt| {
                    mrt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
        group.bench_with_input(BenchmarkId::new(format!("esmt-{}", DATASET), i), i, |b, i| {
            b.iter_batched(
                || {
                    if DATASET == "imis" {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
                        .set_testset(&data).build_query_test()
                    } else {
                        ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_query_test()
                    }
                }, 
                |esmt| {
                    esmt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

criterion_group!(bench_real, continuous_insert_test, after_insert_test ,delete_test, update_full_test, range_query_test);
// criterion_group!(bench_real, after_insert_test);
criterion_main!(bench_real);