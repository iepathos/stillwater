# Performance Guide

Stillwater claims zero-cost abstractions for validation, effects, and error context. This guide documents how we measure those claims and how to interpret results.

## Running benchmarks locally

```bash
# All benchmark suites (validation, effects, context, parallel)
cargo bench --features async

# Individual suites
cargo bench --bench validation
cargo bench --bench effects --features async
cargo bench --bench context
cargo bench --bench parallel --features async
```

Criterion writes HTML reports under `target/criterion/`. Open any `report/index.html` in a browser for detailed plots.

For a quick smoke run (pass `--sample-size` only to criterion benches, not the library test harness):

```bash
cargo bench --features async \
  --bench validation --bench effects --bench context --bench parallel \
  -- --sample-size 10
```

## Benchmark categories

| Suite | File | Compares |
|-------|------|----------|
| Validation | `benches/validation.rs` | `Validation::all_vec` vs manual `Result` accumulation |
| Effects | `benches/effects.rs` | Combinator chain (`map` + `and_then`) vs hand-written async |
| Context | `benches/context.rs` | `ContextError` layering vs equivalent manual struct |
| Parallel | `benches/parallel.rs` | `par_all` / `par2` vs sequential `run` |

Each suite pairs Stillwater APIs with hand-written equivalents so overhead is measurable, not assumed.

## Expected characteristics

On typical release builds:

- **Validation** — Within a few percent of manual accumulation for success paths; error accumulation adds work proportional to error count in both paths.
- **Effects** — Combinator chains should match manual async code when effects use concrete types (no `.boxed()` in the hot path).
- **Context** — `ContextError` should be within a few percent of a manual `Vec<String>` trail; both allocate context strings.
- **Parallel** — `par_all` on cheap `pure` effects measures scheduling overhead; use I/O-heavy effects locally to validate real speedup.

The project success metric is **within 5% of hand-written equivalent code** on CPU-bound paths. Absolute numbers vary by machine; compare relative ratios and track trends over time.

## CI integration

The [Benchmarks workflow](https://github.com/iepathos/stillwater/blob/master/.github/workflows/benchmarks.yml):

- On **pull requests**: runs benchmarks with reduced samples as a compile-and-run smoke test.
- On **push to master**: runs full benchmarks and stores results via [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) for historical comparison.

Regressions are detected by comparing new results to the stored baseline on the default branch.

## Zero-cost verification

Compile-time checks live in `src/effect/combinators/zero_cost_tests.rs` (combinator struct sizes). Runtime benchmarks validate that those abstractions do not add unexpected overhead in hot paths.

Practices for zero-cost usage:

1. Prefer concrete effect types; call `.boxed()` only at collection boundaries (`par_all`, heterogeneous vectors).
2. Use `Validation::all_vec` / tuple `validate_all` for accumulation instead of fail-fast `?` when you need every error.
3. Add `ContextError` at boundaries, not inside tight inner loops.

## Updating this document

After significant API changes, re-run `cargo bench --features async` on release mode and note any ratio shifts in PR descriptions. Do not commit machine-specific timings here; rely on CI history for regression tracking.
