---
number: 037
title: Writer Effect (Logging/Accumulation)
category: foundation
priority: high
status: draft
dependencies: [024]
created: 2025-12-20
---

# Specification 037: Writer Effect (Logging/Accumulation)

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect Trait)

## Context

### The Problem

Collecting logs, metrics, or audit trails alongside computation requires threading state manually through every function call:

```rust
// Without Writer - threading logs manually
fn process(x: i32, logs: &mut Vec<String>) -> Result<i32, Error> {
    logs.push("Starting process".to_string());
    let y = step1(x, logs)?;
    logs.push(format!("Step 1 result: {}", y));
    let z = step2(y, logs)?;
    logs.push(format!("Step 2 result: {}", z));
    Ok(z)
}
```

This approach has several drawbacks:

1. **Threading State Pollution** - Every function must accept and pass through the log accumulator
2. **Coupling** - Functions that don't log still need the log parameter to pass it through
3. **Easy to Forget** - Missing log parameters causes compilation errors scattered throughout the codebase
4. **Testing Difficulty** - Tests need to create and pass log accumulators even when not testing logging

### The Solution

The Writer Effect provides automatic accumulation that composes with Stillwater's existing Effect system:

```rust
// With Writer - automatic accumulation
fn process(x: i32) -> impl WriterEffect<Output = i32, Writes = Vec<String>> {
    tell("Starting process")
        .and_then(|_| step1(x))
        .tap_tell(|y| format!("Step 1 result: {}", y))
        .and_then(step2)
        .tap_tell(|z| format!("Step 2 result: {}", z))
}

// Run and collect logs
let (result, logs) = process(42).run_writer(&env).await;
```

### What It Enables

- **Accumulate logs/metrics without threading state** - Clean function signatures
- **Combine with Effect's Reader and Error** - Full integration with existing combinators
- **Monoid-based accumulation** - Works with any `W: Monoid`, not just `Vec<String>`
- **Type-safe log types** - Different effects can use different log types
- **Zero-cost when unused** - No runtime overhead if logs aren't collected

### Philosophy Alignment

From PHILOSOPHY.md:
- *"Composition over complexity"* - Writer composes with existing Effect combinators
- *"Types guide, don't restrict"* - Monoid constraint enables flexible accumulation strategies
- *"Zero-cost abstractions"* - Concrete types, no boxing for Writer infrastructure

### Prior Art

- **Haskell MTL**: `WriterT` monad transformer with `tell`, `censor`, `listen`
- **Scala ZIO**: `ZIO.log`, `ZIO.logSpan` with integrated logging
- **Cats Effect**: `Writer` monad with monoid-based accumulation
- **Rust log ecosystem**: Runtime logging (this is compile-time effect accumulation)

## Objective

Add a Writer capability to Stillwater's Effect system that:

1. Enables accumulating values of type `W: Monoid` alongside computation
2. Integrates seamlessly with existing Effect combinators (`map`, `and_then`, etc.)
3. Maintains zero-cost abstraction (concrete types, no boxing)
4. Composes with Reader (environment access) and Error (failure handling)
5. Provides ergonomic combinators for common patterns

## Requirements

### Functional Requirements

#### FR-1: WriterEffect Trait

Define a trait extending Effect with a `Writes` associated type:

```rust
/// An effect that accumulates values alongside computation.
pub trait WriterEffect: Effect {
    /// The type of values being accumulated (must be a Monoid).
    type Writes: Monoid + Send;
}
```

#### FR-2: Core Operations

##### FR-2.1: `tell` - Emit a Value

```rust
/// Emit a value to be accumulated, producing unit.
///
/// Environment-generic to work with any environment type (e.g., `()`, `RealEnv`, custom envs).
pub fn tell<W, Env>(w: W) -> impl WriterEffect<Output = (), Writes = W, Error = Infallible, Env = Env>
where
    W: Monoid + Send,
    Env: Send,
```

Usage:
```rust
tell(vec!["User logged in".to_string()])
```

##### FR-2.2: `tell_one` - Emit a Single Item

Convenience for single-item collections:

```rust
/// Emit a single item (convenience for `tell(vec![item])`).
///
/// Environment-generic to work with any environment type.
pub fn tell_one<T, Env>(item: T) -> impl WriterEffect<Output = (), Writes = Vec<T>, Error = Infallible, Env = Env>
where
    T: Send,
    Env: Send,
```

Usage:
```rust
tell_one("Starting process".to_string())
```

#### FR-3: Combinator Extensions

Add methods to `WriterEffectExt` for effects that implement `WriterEffect`:

##### FR-3.1: `tap_tell` - Log After Success

```rust
/// After this effect succeeds, emit a derived value.
fn tap_tell<F, W2>(self, f: F) -> TapTell<Self, F>
where
    F: FnOnce(&Self::Output) -> W2 + Send,
    W2: Into<Self::Writes>;
```

Usage:
```rust
fetch_user(id)
    .tap_tell(|user| vec![format!("Fetched user: {}", user.name)])
```

##### FR-3.2: `censor` - Transform Accumulated Writes

```rust
/// Transform the accumulated writes.
fn censor<F>(self, f: F) -> Censor<Self, F>
where
    F: FnOnce(Self::Writes) -> Self::Writes + Send;
```

Usage:
```rust
effect.censor(|logs| logs.into_iter().filter(|l| !l.contains("DEBUG")).collect())
```

##### FR-3.3: `listen` - Access Accumulated Writes

```rust
/// Include accumulated writes in the output.
fn listen(self) -> Listen<Self>
where
    Self: Sized;
```

Usage:
```rust
let effect = compute().listen();
// Returns (original_output, accumulated_writes)
```

##### FR-3.4: `pass` - Compute a Write Transformation

```rust
/// Use output to determine how to transform writes.
fn pass<F>(self) -> Pass<Self>
where
    Self: WriterEffect<Output = (T, F)>,
    F: FnOnce(Self::Writes) -> Self::Writes + Send;
```

Usage:
```rust
// Output is (value, transformation_fn), writes are transformed
compute()
    .map(|value| (value, |w: Vec<String>| w.into_iter().take(10).collect()))
    .pass()
```

#### FR-4: Execution Methods

##### FR-4.1: `run_writer` - Execute and Collect Writes

```rust
/// Execute the effect and return both result and accumulated writes.
async fn run_writer(self, env: &Self::Env) -> (Result<Self::Output, Self::Error>, Self::Writes);
```

##### FR-4.2: `run_ignore_writes` - Execute and Discard Writes

```rust
/// Execute the effect, discarding accumulated writes (useful for testing or when writes aren't needed).
async fn run_ignore_writes(self, env: &Self::Env) -> Result<Self::Output, Self::Error>;
```

#### FR-5: Lifting Regular Effects

Convert non-Writer effects to WriterEffects with empty writes:

```rust
/// Lift a regular Effect into a WriterEffect with empty writes.
fn into_writer<W: Monoid>(effect: E) -> impl WriterEffect<Output = E::Output, Error = E::Error, Env = E::Env, Writes = W>
```

#### FR-6: Combining Writer Effects

When combining Writer effects, their writes should be combined using `Monoid::combine`:

```rust
// Both effects' logs are combined
let combined = writer_effect_a
    .and_then(|a| writer_effect_b.map(move |b| (a, b)));
// Writes from both are accumulated
```

#### FR-7: Type-Erased Writer Effects

Provide a boxed type alias for storing heterogeneous writer effects:

```rust
/// A type-erased WriterEffect for use in collections, match arms, or recursive functions.
///
/// Similar to `BoxedEffect`, this enables:
/// - Storing different writer effect types in `Vec` or `HashMap`
/// - Returning different effect types from match arms
/// - Breaking infinite types in recursive functions
pub type BoxedWriterEffect<T, E, Env, W> = Box<
    dyn WriterEffect<Output = T, Error = E, Env = Env, Writes = W> + Send
>;

/// Extension trait method for boxing writer effects.
fn boxed_writer(self) -> BoxedWriterEffect<Self::Output, Self::Error, Self::Env, Self::Writes>
where
    Self: Sized + Send + 'static;
```

Usage:
```rust
// Store heterogeneous writer effects
let effects: Vec<BoxedWriterEffect<i32, String, (), Vec<String>>> = vec![
    tell_one("log 1".into()).map(|_| 1).boxed_writer(),
    tell_one("log 2".into()).map(|_| 2).boxed_writer(),
];

// Match arms with different types
fn conditional_log(flag: bool) -> BoxedWriterEffect<i32, String, (), Vec<String>> {
    if flag {
        tell_one("enabled".into()).map(|_| 1).boxed_writer()
    } else {
        pure(0).into_writer().boxed_writer()
    }
}
```

#### FR-8: Concurrent Write Accumulation

When writer effects execute concurrently (via `zip` or parallel combinators), writes are accumulated in **left-to-right order**:

```rust
// Sequential: writes from effect_a, then effect_b
let sequential = effect_a.and_then(|a| effect_b.map(move |b| (a, b)));

// Concurrent: same order - writes from left operand first, then right
let concurrent = effect_a.zip(effect_b);
// Writes: [a_writes..., b_writes...]
```

This ensures deterministic, predictable write ordering regardless of actual execution timing.

#### FR-9: Collection Combinators

Provide combinators for traversing collections with accumulated writes:

```rust
/// Traverse a collection, running a writer effect for each item and accumulating all writes.
pub fn traverse_writer<T, U, E, Env, W, F, Eff>(
    items: Vec<T>,
    f: F,
) -> impl WriterEffect<Output = Vec<U>, Error = E, Env = Env, Writes = W>
where
    T: Send,
    U: Send,
    E: Send,
    Env: Send,
    W: Monoid + Send,
    F: Fn(T) -> Eff + Send,
    Eff: WriterEffect<Output = U, Error = E, Env = Env, Writes = W> + Send;

/// Fold a collection with a writer effect, accumulating writes at each step.
pub fn fold_writer<T, A, E, Env, W, F, Eff>(
    items: Vec<T>,
    init: A,
    f: F,
) -> impl WriterEffect<Output = A, Error = E, Env = Env, Writes = W>
where
    T: Send,
    A: Send,
    E: Send,
    Env: Send,
    W: Monoid + Send,
    F: Fn(A, T) -> Eff + Send,
    Eff: WriterEffect<Output = A, Error = E, Env = Env, Writes = W> + Send;
```

Usage:
```rust
// Analyze multiple files, accumulating audit events from all
let effect = traverse_writer(paths, |path| analyze_with_audit(path));
let (results, all_events) = effect.run_writer(&env).await;
```

### Non-Functional Requirements

#### NFR-1: Zero-Cost Abstractions

- Each combinator must return a concrete type
- No heap allocation for combinator creation
- Writes accumulation should be as efficient as manual accumulation

#### NFR-2: Type Safety

- `Writes` type must implement `Monoid`
- Incompatible write types must fail at compile time
- Error messages should be clear about monoid requirements

#### NFR-3: Ergonomics

- Common patterns (logging strings) should be one-liners
- Works naturally with existing Effect chains
- Minimal boilerplate for typical use cases

## Acceptance Criteria

### Core Functionality

- [ ] **AC1**: `WriterEffect` trait defined with `Writes: Monoid` associated type
- [ ] **AC2**: `tell<W: Monoid>(w: W)` function creates a write-only effect
- [ ] **AC3**: `tell_one<T>(item: T)` convenience function for single items
- [ ] **AC4**: `run_writer` method returns `(Result<T, E>, W)` tuple
- [ ] **AC5**: Writes are accumulated across `and_then` chains
- [ ] **AC6**: Writes are accumulated across `zip` combinations

### Extension Combinators

- [ ] **AC7**: `tap_tell` logs after success, passes value through
- [ ] **AC8**: `censor` transforms accumulated writes
- [ ] **AC9**: `listen` includes writes in output as `(T, W)`
- [ ] **AC10**: `pass` applies output-derived transformation to writes

### Integration

- [ ] **AC11**: Works with `map` - writes unchanged
- [ ] **AC12**: Works with `and_then` - writes combine via Monoid
- [ ] **AC13**: Works with `or_else` - writes from successful branch kept
- [ ] **AC14**: Works with environment access (`asks`)
- [ ] **AC15**: Works with `boxed()` for type erasure
- [ ] **AC16**: Works with custom environment types (not just `()`)
- [ ] **AC17**: `BoxedWriterEffect` type alias available for type erasure
- [ ] **AC18**: `zip` combines writes in left-to-right order

### Monoid Requirements

- [ ] **AC19**: `Vec<T>` works as writes type
- [ ] **AC20**: `String` works as writes type
- [ ] **AC21**: Custom monoid types work
- [ ] **AC22**: `Sum<i32>` works for numeric accumulation (counts, metrics)

### Error Handling

- [ ] **AC23**: On error, accumulated writes up to error point are returned
- [ ] **AC24**: `or_else` recovery preserves original writes

### Collection Combinators

- [ ] **AC25**: `traverse_writer` processes items and accumulates writes from all
- [ ] **AC26**: `fold_writer` reduces with accumulated writes at each step

## Technical Details

### Implementation Approach

#### WriterEffect Trait Definition

```rust
// src/effect/writer/trait_def.rs

use crate::Monoid;
use crate::effect::trait_def::Effect;

/// An effect that accumulates values of type `Writes` alongside computation.
///
/// The `Writes` type must be a `Monoid` to support:
/// - Empty writes (`Monoid::empty()`) for effects that don't write
/// - Combining writes (`Semigroup::combine`) when chaining effects
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
///
/// fn logged_operation() -> impl WriterEffect<Output = i32, Writes = Vec<String>, Error = String, Env = ()> {
///     tell_one("Starting operation".to_string())
///         .and_then(|_| pure(42))
///         .tap_tell(|result| format!("Got result: {}", result))
/// }
/// ```
pub trait WriterEffect: Effect {
    /// The type of values being accumulated.
    ///
    /// Must implement `Monoid` for identity and combination.
    type Writes: Monoid + Send;

    /// Execute this effect and return both result and accumulated writes.
    fn run_writer(self, env: &Self::Env)
        -> impl std::future::Future<Output = (Result<Self::Output, Self::Error>, Self::Writes)> + Send;
}
```

#### Tell Combinator

```rust
// src/effect/writer/tell.rs

use crate::Monoid;
use crate::effect::writer::WriterEffect;
use std::convert::Infallible;
use std::marker::PhantomData;

/// An effect that only emits a value, producing unit.
///
/// Note: No `Clone` bound required - the writes are consumed on execution.
#[derive(Debug)]
pub struct Tell<W, Env> {
    writes: W,
    _env: PhantomData<Env>,
}

impl<W, Env> Clone for Tell<W, Env>
where
    W: Clone,
{
    fn clone(&self) -> Self {
        Self {
            writes: self.writes.clone(),
            _env: PhantomData,
        }
    }
}

impl<W, Env> Effect for Tell<W, Env>
where
    W: Monoid + Send,
    Env: Send,
{
    type Output = ();
    type Error = Infallible;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

impl<W, Env> WriterEffect for Tell<W, Env>
where
    W: Monoid + Send,
    Env: Send,
{
    type Writes = W;

    async fn run_writer(self, _env: &Self::Env) -> (Result<(), Infallible>, W) {
        (Ok(()), self.writes)
    }
}

/// Emit a value to be accumulated.
///
/// Environment-generic: works with any `Env` type.
pub fn tell<W, Env>(w: W) -> Tell<W, Env>
where
    W: Monoid + Send,
    Env: Send,
{
    Tell { writes: w, _env: PhantomData }
}

/// Emit a single item to a Vec accumulator.
///
/// Environment-generic: works with any `Env` type.
pub fn tell_one<T, Env>(item: T) -> Tell<Vec<T>, Env>
where
    T: Send,
    Env: Send,
{
    Tell { writes: vec![item], _env: PhantomData }
}
```

#### TapTell Combinator

```rust
// src/effect/writer/tap_tell.rs

use crate::Monoid;
use crate::effect::writer::WriterEffect;
use std::marker::PhantomData;

/// An effect that emits a derived value after the inner effect succeeds.
#[derive(Debug)]
pub struct TapTell<E, F> {
    inner: E,
    f: F,
}

impl<E, F, W2> Effect for TapTell<E, F>
where
    E: WriterEffect,
    E::Output: Clone + Send,
    F: FnOnce(&E::Output) -> W2 + Send,
    W2: Into<E::Writes>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.inner.run(env).await
    }
}

impl<E, F, W2> WriterEffect for TapTell<E, F>
where
    E: WriterEffect,
    E::Output: Clone + Send,
    F: FnOnce(&E::Output) -> W2 + Send,
    W2: Into<E::Writes>,
{
    type Writes = E::Writes;

    async fn run_writer(self, env: &Self::Env) -> (Result<Self::Output, Self::Error>, Self::Writes) {
        let (result, mut writes) = self.inner.run_writer(env).await;

        if let Ok(ref value) = result {
            let additional: E::Writes = (self.f)(value).into();
            writes = writes.combine(additional);
        }

        (result, writes)
    }
}
```

#### AndThen for WriterEffect

The key insight is that `and_then` on WriterEffects must combine writes from both effects:

```rust
// src/effect/writer/and_then.rs

impl<E, F, E2> WriterEffect for WriterAndThen<E, F, E2>
where
    E: WriterEffect,
    F: FnOnce(E::Output) -> E2 + Send,
    E2: WriterEffect<Error = E::Error, Env = E::Env, Writes = E::Writes>,
{
    type Writes = E::Writes;

    async fn run_writer(self, env: &Self::Env) -> (Result<E2::Output, E::Error>, Self::Writes) {
        let (result1, writes1) = self.inner.run_writer(env).await;

        match result1 {
            Ok(value) => {
                let next_effect = (self.f)(value);
                let (result2, writes2) = next_effect.run_writer(env).await;
                (result2, writes1.combine(writes2))
            }
            Err(e) => (Err(e), writes1),
        }
    }
}
```

### Module Structure

```
src/effect/writer/
├── mod.rs              # Module exports
├── trait_def.rs        # WriterEffect trait
├── tell.rs             # tell, tell_one functions
├── tap_tell.rs         # TapTell combinator
├── censor.rs           # Censor combinator
├── listen.rs           # Listen combinator
├── pass.rs             # Pass combinator
├── and_then.rs         # WriterAndThen combinator
├── zip.rs              # WriterZip combinator (concurrent accumulation)
├── lift.rs             # into_writer for lifting regular Effects
├── boxed.rs            # BoxedWriterEffect type alias and boxing
├── combinators.rs      # traverse_writer, fold_writer, etc.
└── prelude.rs          # Common imports
```

### Extension Trait

```rust
// src/effect/writer/ext.rs

/// Extension trait for WriterEffect combinators.
pub trait WriterEffectExt: WriterEffect {
    /// Emit a derived value after success.
    fn tap_tell<F, W2>(self, f: F) -> TapTell<Self, F>
    where
        Self: Sized,
        Self::Output: Clone + Send,
        F: FnOnce(&Self::Output) -> W2 + Send,
        W2: Into<Self::Writes>,
    {
        TapTell { inner: self, f }
    }

    /// Transform accumulated writes.
    fn censor<F>(self, f: F) -> Censor<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Writes) -> Self::Writes + Send,
    {
        Censor { inner: self, f }
    }

    /// Include writes in output.
    fn listen(self) -> Listen<Self>
    where
        Self: Sized,
    {
        Listen { inner: self }
    }

    /// Execute and collect writes.
    async fn run_writer(self, env: &Self::Env) -> (Result<Self::Output, Self::Error>, Self::Writes)
    where
        Self: Sized,
    {
        WriterEffect::run_writer(self, env).await
    }

    /// Execute, discarding writes.
    async fn run_ignore_writes(self, env: &Self::Env) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        let (result, _writes) = WriterEffect::run_writer(self, env).await;
        result
    }
}

impl<E: WriterEffect> WriterEffectExt for E {}
```

## Dependencies

### Prerequisites

- Spec 024 (Zero-Cost Effect Trait) - for Effect trait design
- Existing `Monoid` trait in `src/monoid.rs`
- Existing `Semigroup` trait in `src/semigroup.rs`

### Affected Components

- `Effect` trait - may need adjustment for Writer integration
- `EffectExt` - may need additional methods
- Effect prelude - new exports
- Effect module - new writer submodule

### External Dependencies

- None (builds on existing Stillwater infrastructure)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tell_emits_value() {
        let effect = tell(vec!["hello".to_string()]);
        let (result, writes) = effect.run_writer(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(writes, vec!["hello".to_string()]);
    }

    #[tokio::test]
    async fn test_tell_one_convenience() {
        let effect = tell_one("hello".to_string());
        let (result, writes) = effect.run_writer(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(writes, vec!["hello".to_string()]);
    }

    #[tokio::test]
    async fn test_tap_tell_logs_result() {
        let effect = pure::<_, String, ()>(42)
            .into_writer::<Vec<String>>()
            .tap_tell(|n| vec![format!("Result: {}", n)]);

        let (result, writes) = effect.run_writer(&()).await;

        assert_eq!(result, Ok(42));
        assert_eq!(writes, vec!["Result: 42".to_string()]);
    }

    #[tokio::test]
    async fn test_writes_accumulate_across_and_then() {
        let effect = tell_one("step 1".to_string())
            .and_then(|_| tell_one("step 2".to_string()))
            .and_then(|_| tell_one("step 3".to_string()));

        let (result, writes) = effect.run_writer(&()).await;

        assert_eq!(result, Ok(()));
        assert_eq!(writes, vec![
            "step 1".to_string(),
            "step 2".to_string(),
            "step 3".to_string(),
        ]);
    }

    #[tokio::test]
    async fn test_censor_transforms_writes() {
        let effect = tell_one("debug: something".to_string())
            .and_then(|_| tell_one("info: important".to_string()))
            .censor(|logs| logs.into_iter().filter(|l| !l.starts_with("debug")).collect());

        let (_, writes) = effect.run_writer(&()).await;

        assert_eq!(writes, vec!["info: important".to_string()]);
    }

    #[tokio::test]
    async fn test_listen_includes_writes_in_output() {
        let effect = tell_one("logged".to_string())
            .map(|_| 42)
            .listen();

        let (result, writes) = effect.run_writer(&()).await;

        assert_eq!(result, Ok((42, vec!["logged".to_string()])));
        assert_eq!(writes, vec!["logged".to_string()]);
    }

    #[tokio::test]
    async fn test_error_preserves_writes_up_to_failure() {
        let effect = tell_one("before error".to_string())
            .and_then(|_| fail::<(), String, ()>("boom".into()))
            .and_then(|_| tell_one("after error".to_string()));

        let (result, writes) = effect.run_writer(&()).await;

        assert!(result.is_err());
        assert_eq!(writes, vec!["before error".to_string()]);
    }

    #[tokio::test]
    async fn test_with_sum_monoid() {
        use crate::monoid::Sum;

        let effect = tell(Sum(1))
            .and_then(|_| tell(Sum(2)))
            .and_then(|_| tell(Sum(3)));

        let (_, writes) = effect.run_writer(&()).await;

        assert_eq!(writes, Sum(6));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[derive(Clone)]
    struct TestEnv {
        multiplier: i32,
    }

    #[tokio::test]
    async fn test_writer_with_environment() {
        fn compute(x: i32) -> impl WriterEffect<Output = i32, Writes = Vec<String>, Error = String, Env = TestEnv> {
            asks::<_, String, TestEnv, _>(|env| env.multiplier)
                .into_writer()
                .tap_tell(|m| vec![format!("Multiplier: {}", m)])
                .map(move |m| x * m)
                .tap_tell(|result| vec![format!("Result: {}", result)])
        }

        let env = TestEnv { multiplier: 3 };
        let (result, logs) = compute(7).run_writer(&env).await;

        assert_eq!(result, Ok(21));
        assert_eq!(logs, vec![
            "Multiplier: 3".to_string(),
            "Result: 21".to_string(),
        ]);
    }

    #[tokio::test]
    async fn test_mixed_writer_and_regular_effects() {
        // Verify regular effects lift cleanly into writer context
        let effect = pure::<_, String, ()>(10)
            .into_writer::<Vec<String>>()
            .and_then(|n|
                tell_one(format!("Got: {}", n))
                    .map(move |_| n * 2)
            )
            .tap_tell(|result| vec![format!("Final: {}", result)]);

        let (result, logs) = effect.run_writer(&()).await;

        assert_eq!(result, Ok(20));
        assert_eq!(logs, vec!["Got: 10".to_string(), "Final: 20".to_string()]);
    }

    #[tokio::test]
    async fn test_traverse_writer_accumulates_all() {
        let items = vec![1, 2, 3];
        let effect = traverse_writer(items, |n| {
            tell_one::<String, ()>(format!("Processing {}", n))
                .map(move |_| n * 10)
        });

        let (result, logs) = effect.run_writer(&()).await;

        assert_eq!(result, Ok(vec![10, 20, 30]));
        assert_eq!(logs, vec![
            "Processing 1".to_string(),
            "Processing 2".to_string(),
            "Processing 3".to_string(),
        ]);
    }

    #[tokio::test]
    async fn test_zip_combines_writes_left_to_right() {
        let left = tell_one::<String, ()>("left".into()).map(|_| 1);
        let right = tell_one::<String, ()>("right".into()).map(|_| 2);

        let (result, logs) = left.zip(right).run_writer(&()).await;

        assert_eq!(result, Ok((1, 2)));
        assert_eq!(logs, vec!["left".to_string(), "right".to_string()]);
    }

    #[tokio::test]
    async fn test_boxed_writer_in_collection() {
        let effects: Vec<BoxedWriterEffect<i32, String, (), Vec<String>>> = vec![
            tell_one("a".into()).map(|_| 1).boxed_writer(),
            tell_one("b".into()).map(|_| 2).boxed_writer(),
            tell_one("c".into()).map(|_| 3).boxed_writer(),
        ];

        let mut results = Vec::new();
        let mut all_logs = Vec::new();

        for effect in effects {
            let (result, logs) = effect.run_writer(&()).await;
            results.push(result.unwrap());
            all_logs.extend(logs);
        }

        assert_eq!(results, vec![1, 2, 3]);
        assert_eq!(all_logs, vec!["a", "b", "c"]);
    }
}
```

### Custom Environment Tests (Debtmap-style)

```rust
#[cfg(test)]
mod custom_env_tests {
    use super::*;

    /// Trait representing an analysis environment (similar to debtmap::AnalysisEnv)
    trait AnalysisEnv {
        fn config(&self) -> &Config;
    }

    #[derive(Clone)]
    struct Config {
        threshold: u32,
    }

    #[derive(Clone)]
    struct RealEnv {
        config: Config,
    }

    impl AnalysisEnv for RealEnv {
        fn config(&self) -> &Config {
            &self.config
        }
    }

    /// Helper to query config (similar to debtmap's asks_config)
    fn asks_config<U, Env, F>(f: F) -> impl Effect<Output = U, Error = String, Env = Env>
    where
        Env: AnalysisEnv + Clone + Send + Sync,
        F: Fn(&Config) -> U + Send + Sync + 'static,
        U: Send + 'static,
    {
        asks(move |env: &Env| f(env.config()))
    }

    #[derive(Debug, Clone, PartialEq)]
    enum AuditEvent {
        Started,
        ThresholdUsed(u32),
        Completed(i32),
    }

    #[tokio::test]
    async fn test_writer_with_custom_env() {
        fn analyze<Env>(value: i32) -> impl WriterEffect<
            Output = i32,
            Error = String,
            Env = Env,
            Writes = Vec<AuditEvent>,
        >
        where
            Env: AnalysisEnv + Clone + Send + Sync + 'static,
        {
            tell_one(AuditEvent::Started)
                .and_then(move |_| {
                    asks_config::<u32, Env, _>(|cfg| cfg.threshold)
                        .into_writer()
                        .tap_tell(|t| vec![AuditEvent::ThresholdUsed(*t)])
                        .map(move |t| if value > t as i32 { value } else { t as i32 })
                })
                .tap_tell(|result| vec![AuditEvent::Completed(*result)])
        }

        let env = RealEnv {
            config: Config { threshold: 10 },
        };

        let (result, events) = analyze::<RealEnv>(15).run_writer(&env).await;

        assert_eq!(result, Ok(15));
        assert_eq!(events, vec![
            AuditEvent::Started,
            AuditEvent::ThresholdUsed(10),
            AuditEvent::Completed(15),
        ]);
    }
}
```

### Property Tests

```rust
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_tell_emits_exactly_what_given(logs: Vec<String>) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let effect = tell(logs.clone());
                let (_, writes) = effect.run_writer(&()).await;
                prop_assert_eq!(writes, logs);
            })
        }

        #[test]
        fn prop_censor_identity_preserves_writes(logs: Vec<String>) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let effect = tell(logs.clone()).censor(|w| w);
                let (_, writes) = effect.run_writer(&()).await;
                prop_assert_eq!(writes, logs);
            })
        }

        #[test]
        fn prop_writes_combine_associatively(a: Vec<String>, b: Vec<String>, c: Vec<String>) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // (a.b).c
                let left = tell(a.clone())
                    .and_then(|_| tell(b.clone()))
                    .and_then(|_| tell(c.clone()));

                // a.(b.c) - same because and_then is sequential
                let right = tell(a.clone())
                    .and_then(|_| tell(b.clone()).and_then(|_| tell(c.clone())));

                let (_, left_writes) = left.run_writer(&()).await;
                let (_, right_writes) = right.run_writer(&()).await;

                prop_assert_eq!(left_writes, right_writes);
            })
        }
    }
}
```

## Documentation Requirements

### Code Documentation

Each combinator should have comprehensive rustdoc:

```rust
/// Emit a value to be accumulated alongside computation.
///
/// `tell` is the fundamental Writer operation. It produces unit (`()`) as output
/// but emits a value that will be accumulated with other writes in the effect chain.
///
/// # Type Parameters
///
/// * `W` - The type to accumulate. Must implement `Monoid` for combination.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::writer::prelude::*;
///
/// // Simple logging
/// let effect = tell(vec!["Starting".to_string()]);
/// let ((), logs) = effect.run_writer(&()).await;
/// assert_eq!(logs, vec!["Starting".to_string()]);
/// ```
///
/// # Chaining Writes
///
/// ```rust
/// let effect = tell(vec!["Step 1".into()])
///     .and_then(|_| tell(vec!["Step 2".into()]));
/// let ((), logs) = effect.run_writer(&()).await;
/// assert_eq!(logs, vec!["Step 1", "Step 2"]);
/// ```
///
/// # With Different Monoids
///
/// ```rust
/// use stillwater::monoid::Sum;
///
/// // Count operations
/// let effect = tell(Sum(1))
///     .and_then(|_| tell(Sum(1)))
///     .and_then(|_| tell(Sum(1)));
/// let ((), Sum(count)) = effect.run_writer(&()).await;
/// assert_eq!(count, 3);
/// ```
```

### User Documentation

Add to the Effect guide:

```markdown
## Writer Effect: Accumulating Logs and Metrics

The Writer Effect enables accumulating values alongside computation without
threading state through every function.

### Basic Usage

```rust
use stillwater::effect::writer::prelude::*;

// Log steps in a computation
fn process(input: i32) -> impl WriterEffect<Output = i32, Writes = Vec<String>> {
    tell_one("Starting process")
        .and_then(move |_| pure(input * 2))
        .tap_tell(|result| format!("Doubled to: {}", result))
        .and_then(|n| pure(n + 10))
        .tap_tell(|result| format!("Final result: {}", result))
}

#[tokio::main]
async fn main() {
    let (result, logs) = process(5).run_writer(&()).await;

    println!("Result: {:?}", result);  // Ok(20)
    for log in logs {
        println!("  - {}", log);
    }
    // Prints:
    //   - Starting process
    //   - Doubled to: 10
    //   - Final result: 20
}
```

### Key Operations

| Operation | Purpose | Example |
|-----------|---------|---------|
| `tell(w)` | Emit a value | `tell(vec!["message".into()])` |
| `tell_one(item)` | Emit single item | `tell_one("message".into())` |
| `tap_tell(f)` | Log after success | `.tap_tell(\|x\| format!("{}", x))` |
| `censor(f)` | Transform writes | `.censor(\|logs\| filter(logs))` |
| `listen()` | Include writes in output | `.listen()` → `(value, writes)` |

### Using Different Monoids

Writer works with any `Monoid`, not just `Vec<String>`:

```rust
use stillwater::monoid::Sum;

// Count database operations
fn fetch_all_users() -> impl WriterEffect<Output = Vec<User>, Writes = Sum<u32>> {
    tell(Sum(1))  // Count this operation
        .and_then(|_| fetch_page(1))
        .and_then(|users|
            tell(Sum(1))
                .and_then(|_| fetch_page(2))
                .map(|more| [users, more].concat())
        )
}

let (users, Sum(db_calls)) = fetch_all_users().run_writer(&env).await;
println!("Made {} database calls", db_calls);  // 2
```

### Integration with Reader and Error

Writer composes naturally with Stillwater's other effects:

```rust
fn audit_operation(user_id: i32)
    -> impl WriterEffect<Output = User, Error = AppError, Env = AppEnv, Writes = Vec<AuditLog>>
{
    asks::<_, AppError, AppEnv, _>(|env| env.db.clone())
        .into_writer()
        .and_then(move |db|
            db.fetch_user(user_id)
                .into_writer()
                .tap_tell(|user| AuditLog::access(user_id, "fetch"))
        )
        .map_err(AppError::from)
}
```

### Integration with Custom Environments (e.g., Debtmap)

For packages with custom environment types like `debtmap::RealEnv`, Writer integrates seamlessly:

```rust
use debtmap::env::{AnalysisEnv, RealEnv};
use debtmap::errors::AnalysisError;
use stillwater::effect::writer::prelude::*;

/// Audit event for tracking analysis operations.
#[derive(Debug, Clone)]
pub enum AuditEvent {
    Started { path: PathBuf },
    ThresholdChecked { name: String, value: u32 },
    Completed { complexity: u32, duration_ms: u64 },
    Warning { message: String },
}

/// Analyze a file with full audit trail.
fn analyze_with_audit<Env>(
    path: PathBuf,
) -> impl WriterEffect<Output = FileMetrics, Error = AnalysisError, Env = Env, Writes = Vec<AuditEvent>>
where
    Env: AnalysisEnv + Clone + Send + Sync + 'static,
{
    // Start with audit event
    tell_one(AuditEvent::Started { path: path.clone() })
        // Query config using Reader pattern
        .and_then(move |_| {
            asks_config::<Option<u32>, Env, _>(|cfg| {
                cfg.thresholds.as_ref().and_then(|t| t.complexity)
            })
            .into_writer()
        })
        // Log threshold if present
        .tap_tell(|threshold| {
            threshold.map(|t| AuditEvent::ThresholdChecked {
                name: "complexity".into(),
                value: t,
            })
        })
        // Perform analysis
        .and_then(move |threshold| {
            analyze_file_effect(path)
                .into_writer()
                .tap_tell(move |metrics| AuditEvent::Completed {
                    complexity: metrics.complexity,
                    duration_ms: metrics.duration.as_millis() as u64,
                })
        })
}

/// Batch analysis with accumulated diagnostics.
fn analyze_batch<Env>(
    paths: Vec<PathBuf>,
) -> impl WriterEffect<Output = Vec<FileMetrics>, Error = AnalysisError, Env = Env, Writes = Vec<AuditEvent>>
where
    Env: AnalysisEnv + Clone + Send + Sync + 'static,
{
    // Traverse with accumulated writes from each file
    traverse_writer(paths, analyze_with_audit)
}

/// Run and collect both results and audit trail.
async fn run_analysis(paths: Vec<PathBuf>, config: DebtmapConfig) -> anyhow::Result<()> {
    let env = RealEnv::new(config);
    let (result, audit_trail) = analyze_batch(paths).run_writer(&env).await;

    // Process audit trail (send to monitoring, write to file, etc.)
    for event in &audit_trail {
        match event {
            AuditEvent::Warning { message } => log::warn!("{}", message),
            AuditEvent::Completed { complexity, .. } if *complexity > 50 => {
                log::info!("High complexity file analyzed: {}", complexity);
            }
            _ => {}
        }
    }

    result.map(|_| ()).map_err(Into::into)
}
```

### Coexistence with ProgressSink

Writer and ProgressSink serve complementary purposes:

| Aspect | Writer Effect | ProgressSink |
|--------|--------------|--------------|
| **Purpose** | Structured data accumulation | Real-time UI feedback |
| **Output** | Returned with result | Side-effectful during execution |
| **Use case** | Audit logs, metrics, diagnostics | Progress bars, spinners, status |
| **Testability** | Pure, check output value | Mock sink, verify calls |

They can be used together:

```rust
fn analyze_with_both<Env>(path: PathBuf) -> impl WriterEffect<...>
where
    Env: AnalysisEnv + HasProgress + Clone + Send + Sync,
{
    // Progress for real-time UI (side-effect)
    with_stage("Analyzing",
        // Writer for structured audit (accumulated)
        tell_one(AuditEvent::Started { path: path.clone() })
            .and_then(move |_| analyze_file_effect(path).into_writer())
            .tap_tell(|m| AuditEvent::Completed { complexity: m.complexity })
    )
}
```

## Implementation Notes

### Error Type for `tell`

The `tell` function never fails, so its error type should be `Infallible` (or the never type `!` when stable). This enables composition with any error type through the `From<Infallible>` blanket impl.

### Lazy vs Eager Accumulation

Writes are accumulated eagerly as effects execute. This is simpler and matches user expectations. Lazy accumulation would be more complex and offer little benefit for typical use cases.

### Performance Considerations

- Accumulation happens during execution, not combinator construction
- `Vec::combine` (extend) has amortized O(1) append per element
- For high-performance logging, consider a `Vec<LogEntry>` with pre-allocated capacity
- Alternative: use a monoid wrapper around a pre-sized buffer

### Future Extensions

- `Traced` variant with automatic span nesting for structured logs
- Integration with `tracing` crate for async-aware spans
- `WriterT` transformer for stacking with other effects

## Migration and Compatibility

### Backward Compatibility

Purely additive - no breaking changes to existing Effect code.

### Migration Path

Existing code using manual log threading can adopt Writer incrementally:

```rust
// Before: manual threading
fn process(x: i32, logs: &mut Vec<String>) -> Result<i32, Error> {
    logs.push("Starting".into());
    let y = step(x)?;
    logs.push(format!("Result: {}", y));
    Ok(y)
}

// After: Writer effect
fn process(x: i32) -> impl WriterEffect<Output = i32, Writes = Vec<String>, Error = Error, Env = ()> {
    tell_one("Starting".to_string())
        .and_then(move |_| step(x).into_writer())
        .tap_tell(|y| format!("Result: {}", y))
}
```

---

*"Writing is easy. You just stare at the blank page until drops of blood form on your forehead."* — Gene Fowler (but with Writer, the blood drops accumulate automatically)
