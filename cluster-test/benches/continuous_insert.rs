use criterion::{Criterion, criterion_group, criterion_main};

pub fn continuous_insert_test(c: &mut Criterion) {
    
}

criterion_group!(continuous_insert, continuous_insert_test);
criterion_main!(continuous_insert);