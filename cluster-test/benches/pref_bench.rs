use std::{path::PathBuf, str::FromStr};
use cluster_test::{utils::{MRTreeBuilder, ESMTreeBuilder}, read_dataset};
use criterion::{Criterion, criterion_group, criterion_main};
extern crate bench_pref;
use bench_pref::FlamegraphProfiler;

pub fn delete_flamegraph_test(c: &mut Criterion) {
    let data = read_dataset("uniform", PathBuf::from_str("/home/youya/ESMT/target/release/data_set/uniform/uniform.txt").unwrap()).unwrap();
    let mut group = c.benchmark_group("construct flame");
    group.bench_function("mrt", |b| {
        b.iter_batched(
            || {
                MRTreeBuilder::new().base_size(10000).set_testset(&data).build_delete_test()
            }, 
            |mrt| {
                mrt.exec()
            }, 
            criterion::BatchSize::PerIteration);
    });
    group.bench_function("esmt-h6-8000", |b| {
        b.iter_batched(
            || {
                ESMTreeBuilder::new().base_size(8000)
                    .range([0.0, 0.0], [160.0, 160.0])
                    .set_testset(&data).build_insert_test()
            }, 
            |esmt| {
                esmt.exec()
            }, 
            criterion::BatchSize::PerIteration);
    });
    group.finish();
}

criterion_group! {
    name = mybench_pref;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(100)).sample_size(20);
    targets = delete_flamegraph_test
}
// criterion_group!(mybench, update_full_test);
criterion_main!(mybench_pref);