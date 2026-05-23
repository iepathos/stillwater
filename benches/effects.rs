//! Effect composition vs hand-written async call benchmarks.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use stillwater::effect::prelude::*;

const CHAIN_DEPTH: usize = 8;

/// Build a chain of `map` + `and_then` combinators (fixed depth for a stable concrete type).
fn stillwater_chain() -> impl Effect<Output = i32, Error = (), Env = ()> {
    pure::<_, (), ()>(0i32)
        .map(|x| x + 1)
        .and_then(|x| pure(x * 2))
        .map(|x| x + 2)
        .and_then(|x| pure(x * 2))
        .map(|x| x + 3)
        .and_then(|x| pure(x * 2))
        .map(|x| x + 4)
        .and_then(|x| pure(x * 2))
        .map(|x| x + 5)
        .and_then(|x| pure(x * 2))
        .map(|x| x + 6)
        .and_then(|x| pure(x * 2))
        .map(|x| x + 7)
        .and_then(|x| pure(x * 2))
        .map(|x| x + CHAIN_DEPTH as i32)
        .and_then(|x| pure(x * 2))
}

/// Equivalent hand-written async function chain.
async fn manual_chain() -> Result<i32, ()> {
    let mut value = 0i32;
    for i in 1..=CHAIN_DEPTH {
        value += i as i32;
        value *= 2;
    }
    Ok(value)
}

fn effect_vs_manual(c: &mut Criterion) {
    let mut group = c.benchmark_group("effects");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    group.bench_function("stillwater_chain", |b| {
        b.to_async(&rt).iter_batched(
            stillwater_chain,
            |effect| async move { effect.execute(black_box(&())).await },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("manual_chain", |b| {
        b.to_async(&rt).iter(manual_chain);
    });

    group.finish();
}

criterion_group!(benches, effect_vs_manual);
criterion_main!(benches);
