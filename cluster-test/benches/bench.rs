use std::{path::PathBuf, str::FromStr};
use cluster_test::{read_dataset, utils::{MRTreeBuilder, ESMTreeBuilder}};
use criterion::{Criterion, criterion_group, criterion_main, PlotConfiguration, BenchmarkId};

pub fn continuous_insert_test(c: &mut Criterion) {
    let data = read_dataset("imis", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/imis3days_0/imis_compacted.txt").unwrap()).unwrap();
    let mut group = c.benchmark_group("Continuous-Insert");
    group.plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));
    // for i in [1000usize, 2000, 4000, 8000, 16000, 32000, 64000, 128000, 256000, 512000, 1024000].iter() {
    for i in [1000usize, 2000, 4000].iter() {
        group.bench_with_input(BenchmarkId::new("mrt", i), i, |b, i| {
            b.iter_batched(
                || {
                    MRTreeBuilder::new().base_size(*i).set_testset(&data).build_insert_test()
                }, 
                |mrt| {
                    mrt.exec()
                }, 
                criterion::BatchSize::PerIteration);
        });
        group.bench_with_input(BenchmarkId::new("esmt", i), i, |b, i| {
            b.iter_batched(
                || {
                    ESMTreeBuilder::new().base_size(*i)
                        .range([20.9999999936125, 35.0000449930892], [28.9999499908944, 38.9999999852576])
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

criterion_group!(mybench, continuous_insert_test);
criterion_main!(mybench);