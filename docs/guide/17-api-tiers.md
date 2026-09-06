# API Tiers and Composition Decisions

Stillwater keeps a broad API, but not every module belongs in the default mental model.
Use the smallest tier that solves the application problem.

## Core

Start here for most applications:

- `Validation` and `NonEmptyVec` for independent checks and accumulated errors.
- `Effect`, free constructors, and essential combinators for imperative boundaries.
- Reader helpers (`ask`, `asks`, `local`) for explicit dependencies.
- `ContextError` for adding boundary context.
- `.boxed()` and `BoxedEffect` only where type erasure is required.

The recommended architecture is data first: load facts in the shell, pass them to a pure
function that returns a decision or plan, then interpret that plan at the boundary. See the
[`user_registration` example](../../examples/user_registration.rs) for the canonical shape.

## Operational

Reach for these when runtime behavior is part of the requirement:

- Explicit sequential or parallel traversal and sequencing.
- Parallel execution, races, concurrency limits, retry, and timeout.
- Runtime `bracket` helpers for resources whose cleanup is itself effectful.
- `tracing` integration for production diagnostics.

The Cargo feature named `async` enables Tokio-backed retry and timeout helpers. Core
`Effect` composition is always asynchronous and does not require that feature.

## Advanced

These modules remain supported, but should be adopted for a specific need rather than as
the default application architecture:

- `IO` is a narrow, infallible projection helper. Prefer ordinary Effect constructors for
  fallible service calls and do not treat `IO` as a second general effect system.
- `WriterEffect` accumulates typed output as part of a result. Use it when the accumulated
  value is domain data; use `tracing` for operational logs and spans.
- `SinkEffect` streams emissions to a supplied consumer. Use it when values must be handled
  incrementally instead of retained in memory.
- Type-level resource tracking proves acquire/release accounting in composition. Prefer
  ordinary Rust RAII for ownership-bound cleanup and runtime `bracket` when cleanup is
  asynchronous or must be represented in the effect result.

## Decision table

| Need | Default choice |
|------|----------------|
| Accumulate independent validation errors | `Validation` |
| Describe an I/O boundary | `Effect` plus a narrow environment trait |
| Operational logs and spans | `tracing` |
| Typed audit output returned with a value | `WriterEffect` |
| Incremental emissions | `SinkEffect` |
| Ownership-bound cleanup | RAII (`Drop`) |
| Async/fallible cleanup protocol | Runtime `bracket` |
| Compile-time acquire/release accounting | Type-level resource API |
| Sync environment access | `from_fn` |
| Async work owning cheap cloned handles | `from_async` |
| Async work borrowing the environment | `from_async_ref` |

These tiers classify the existing surface for 2.0. They do not introduce feature gates or
move APIs to companion crates.
