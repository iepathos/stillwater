//! Validation vs manual `Result` accumulation benchmarks.

mod support;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use support::{fields, manual_accumulate, stillwater_accumulate};

fn validation_vs_result(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");
    let inputs = fields();

    group.bench_function("stillwater_accumulate", |b| {
        b.iter(|| stillwater_accumulate(black_box(&inputs)));
    });

    group.bench_function("manual_accumulate", |b| {
        b.iter(|| manual_accumulate(black_box(&inputs)));
    });

    group.finish();
}

criterion_group!(benches, validation_vs_result);
criterion_main!(benches);
