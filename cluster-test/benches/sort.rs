use std::{path::PathBuf, str::FromStr};
use cluster_test::{read_dataset, utils::ESMTreeBuilder};
use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId};


pub const SAMPLE_SIZE: usize = 20;

pub fn continuous_insert_test(c: &mut Criterion) {
    let data = read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap();
    let mut group = c.benchmark_group("Continuous-Insert");
    group.sample_size(SAMPLE_SIZE);
    // group.plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [1000usize, 2000, 4000].iter() {
        group.bench_with_input(BenchmarkId::new("esmt-ds", i), i, |b, i| {
            b.iter_batched(
                || {
                    ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_insert_test()
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
    let data = read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap();
    let mut group = c.benchmark_group("Range-query");
    group.sample_size(SAMPLE_SIZE);
    for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    // for i in [1000usize, 2000, 4000].iter() {
        group.bench_with_input(BenchmarkId::new("esmt-ds-query", i), i, |b, i| {
            b.iter_batched(
                || {
                    ESMTreeBuilder::new().base_size(*i)
                        .range([0.0, 0.0], [160.0, 160.0])
                        .set_testset(&data).build_query_test()
                }, 
                |esmt| {
                    esmt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
    }
    group.finish();
}

criterion_group!(bench_sort, continuous_insert_test, range_query_test);
// criterion_group!(bench_real, after_insert_test);
criterion_main!(bench_sort);