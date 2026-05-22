//! ContextError vs plain error wrapping benchmarks.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use stillwater::ContextError;

const CONTEXT_LAYERS: &[&str] = &[
    "parsing response",
    "fetching resource",
    "loading configuration",
    "initializing service",
];

/// Build a `ContextError` with layered context.
fn stillwater_context() -> ContextError<&'static str> {
    let mut err = ContextError::new("connection refused");
    for layer in CONTEXT_LAYERS {
        err = err.context(*layer);
    }
    err
}

/// Manual error type mirroring context accumulation.
#[derive(Clone)]
struct ManualContextError {
    _inner: &'static str,
    context: Vec<String>,
}

impl ManualContextError {
    fn new(error: &'static str) -> Self {
        Self {
            _inner: error,
            context: Vec::new(),
        }
    }

    fn context(mut self, msg: impl Into<String>) -> Self {
        self.context.push(msg.into());
        self
    }

}

fn manual_context() -> ManualContextError {
    let mut err = ManualContextError::new("connection refused");
    for layer in CONTEXT_LAYERS {
        err = err.context(*layer);
    }
    err
}

fn context_vs_plain(c: &mut Criterion) {
    let mut group = c.benchmark_group("context");

    group.bench_function("stillwater_context", |b| {
        b.iter(|| black_box(stillwater_context()));
    });

    group.bench_function("manual_context", |b| {
        b.iter(|| black_box(manual_context()));
    });

    group.finish();
}

criterion_group!(benches, context_vs_plain);
criterion_main!(benches);
