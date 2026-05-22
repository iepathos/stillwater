//! Parallel vs sequential effect execution benchmarks.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stillwater::effect::prelude::*;

const EFFECT_COUNT: usize = 8;

fn build_pure_effects() -> Vec<BoxedEffect<i32, (), ()>> {
    (0..EFFECT_COUNT)
        .map(|i| pure(i as i32).boxed())
        .collect()
}

async fn sequential_all(effects: Vec<BoxedEffect<i32, (), ()>>, env: &()) -> Vec<i32> {
    let mut results = Vec::with_capacity(effects.len());
    for effect in effects {
        results.push(effect.run(env).await.expect("pure effect"));
    }
    results
}

fn parallel_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    group.bench_function("par_all", |b| {
        b.to_async(&rt).iter(|| {
            let effects = build_pure_effects();
            par_all(effects, black_box(&()))
        });
    });

    group.bench_function("sequential", |b| {
        b.to_async(&rt).iter(|| {
            let effects = build_pure_effects();
            sequential_all(effects, black_box(&()))
        });
    });

    group.bench_function("par2_heterogeneous", |b| {
        b.to_async(&rt).iter(|| {
            let e1 = pure::<_, (), ()>(1);
            let e2 = pure::<_, (), ()>(2);
            par2(e1, e2, black_box(&()))
        });
    });

    group.finish();
}

criterion_group!(benches, parallel_vs_sequential);
criterion_main!(benches);
