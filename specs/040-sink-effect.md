---
number: 40
title: Sink Effect for Streaming Output
category: foundation
priority: medium
status: draft
dependencies: [37]
created: 2025-12-20
---

# Specification 040: Sink Effect for Streaming Output

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 037 (Writer Effect)

## Context

### The Problem

The Writer Effect (Spec 037) provides pure, testable accumulation of logs, metrics, and audit trails. However, it accumulates all writes in memory until `run_writer` completes. For long-running processes with high-volume output, this causes:

1. **Unbounded memory growth** - Processing millions of items accumulates millions of log entries
2. **Delayed visibility** - Logs only appear after entire computation finishes
3. **Batch-only semantics** - No streaming to external systems during execution

```rust
// Current Writer Effect - accumulates everything in memory
let effect = traverse_writer(million_items, |item| {
    tell_one(format!("Processing: {}", item))
        .map(|_| process(item))
});

// Memory holds 1M log strings until this returns
let (result, logs) = effect.run_writer(&env).await;
```

### The Solution

A separate `SinkEffect` trait for streaming output that:
- Emits items immediately to a provided sink function
- Keeps memory constant regardless of output volume
- Runs pure Writer in tests, Sink in production
- Maintains clear separation of concerns

```rust
// Sink Effect - streams immediately, constant memory
let effect = traverse_sink(million_items, |item| {
    emit(format!("Processing: {}", item))
        .map(|_| process(item))
});

// Items emitted in real-time, no accumulation
effect.run_with_sink(&env, |log| async {
    send_to_logging_service(log).await
}).await;
```

### Design Philosophy

| Trait | Purpose | Memory | Best For |
|-------|---------|--------|----------|
| WriterEffect | Pure accumulation | O(n) | Testing, short chains, audit trails |
| SinkEffect | Streaming output | O(1) | Production, high-volume, real-time logs |

Both traits can express the same operations but with different execution semantics. Code can be written once and executed with either strategy.

## Objective

Create a `SinkEffect` trait that:

1. Streams items to a sink function during execution
2. Has constant memory usage regardless of output count
3. Provides combinators parallel to WriterEffect
4. Enables clear testing patterns (accumulate in tests, stream in prod)
5. Supports async sinks for network/IO operations
6. Integrates with existing Effect infrastructure

## Requirements

### Functional Requirements

#### 1. SinkEffect Trait

```rust
/// An effect that emits items to a sink during execution.
///
/// Unlike WriterEffect which accumulates writes, SinkEffect streams
/// items immediately to the provided sink function.
pub trait SinkEffect: Effect {
    /// The type of items emitted to the sink.
    type Item: Send;

    /// Execute this effect, emitting items to the sink as they occur.
    ///
    /// The sink function is called for each emitted item. Items are
    /// emitted in order, and the sink can be async for I/O operations.
    fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send;
}
```

#### 2. Core Constructors

- `emit<T, E, Env>(item: T) -> Emit<T, E, Env>` - Emit a single item to sink
- `emit_many<I, T, E, Env>(items: I) -> EmitMany<I, T, E, Env>` - Emit multiple items
- `into_sink<Eff, T>(effect: Eff) -> IntoSink<Eff, T>` - Lift Effect to SinkEffect (no emissions)

#### 3. Combinators

All combinators preserve streaming semantics:

- `SinkAndThen` - Chain dependent effects, stream from both
- `SinkMap` - Transform output value
- `SinkMapErr` - Transform error
- `SinkOrElse` - Error recovery
- `SinkZip` - Combine two effects, stream from both
- `TapEmit` - Emit derived value on success

#### 4. Collection Combinators

- `traverse_sink<I, F, Eff>(items: I, f: F) -> TraverseSink<I, F>` - Process items, streaming output
- `fold_sink<I, F, Eff, Acc>(items: I, init: Acc, f: F) -> FoldSink<I, F, Acc>` - Reduce with streaming

#### 5. Extension Trait

```rust
pub trait SinkEffectExt: SinkEffect {
    /// Emit a derived value after success.
    fn tap_emit<F>(self, f: F) -> TapEmit<Self, F>
    where
        F: FnOnce(&Self::Output) -> Self::Item;

    /// Convert to a boxed SinkEffect for type erasure.
    fn boxed_sink(self) -> BoxedSinkEffect<...>;

    /// Execute with a collecting sink (for testing).
    ///
    /// This is a bridge to Writer semantics - collects all
    /// emitted items into a Vec for assertion.
    async fn run_collecting(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Vec<Self::Item>);
}
```

#### 6. Testing Support

The `run_collecting` method bridges Sink to Writer semantics for testing:

```rust
// Production - stream to external system
effect.run_with_sink(&env, |log| async {
    send_to_remote(log).await
}).await;

// Testing - collect for assertions
let (result, collected) = effect.run_collecting(&env).await;
assert_eq!(collected.len(), 5);
assert!(collected.iter().all(|log| log.contains("processed")));
```

### Non-Functional Requirements

- **Constant memory**: Core operations use O(1) memory regardless of emission count
- **Zero-cost when not emitting**: Effects without emissions compile to same code as Effect
- **Async sink support**: Sinks can perform I/O without blocking
- **Thread-safe**: All types are `Send + Sync` where inner types are
- **No Monoid requirement**: Unlike Writer, no algebraic structure needed on Item type
- **Composable**: Works with existing Effect combinators where possible

## Acceptance Criteria

- [ ] `SinkEffect` trait defined with `Item` type and `run_with_sink` method
- [ ] `emit` constructor creates effect that emits single item
- [ ] `emit_many` constructor creates effect that emits iterator of items
- [ ] `into_sink` lifts regular Effect to SinkEffect with no emissions
- [ ] `SinkAndThen` chains effects, streaming from both
- [ ] `SinkMap` transforms output without affecting emissions
- [ ] `SinkMapErr` transforms error without affecting emissions
- [ ] `SinkOrElse` enables error recovery while preserving emissions
- [ ] `SinkZip` combines effects, streaming from both left-to-right
- [ ] `TapEmit` emits derived value on success
- [ ] `traverse_sink` processes collections with streaming output
- [ ] `fold_sink` reduces collections with streaming output
- [ ] `run_collecting` collects emissions for testing
- [ ] `BoxedSinkEffect` enables type erasure for heterogeneous collections
- [ ] Async sinks work correctly with I/O operations
- [ ] Memory remains constant when processing large collections
- [ ] Comprehensive test suite covering all combinators
- [ ] Documentation with examples for production and testing patterns

## Technical Details

### Module Structure

```
src/effect/sink/
├── mod.rs           # Module exports, SinkEffect trait
├── trait_def.rs     # SinkEffect trait definition
├── ext.rs           # SinkEffectExt extension trait
├── emit.rs          # emit, emit_many constructors
├── into_sink.rs     # into_sink for lifting Effect
├── and_then.rs      # SinkAndThen combinator
├── map.rs           # SinkMap combinator
├── map_err.rs       # SinkMapErr combinator
├── or_else.rs       # SinkOrElse combinator
├── zip.rs           # SinkZip combinator
├── tap_emit.rs      # TapEmit combinator
├── combinators.rs   # traverse_sink, fold_sink
├── boxed.rs         # BoxedSinkEffect for type erasure
├── prelude.rs       # Convenience re-exports
└── tests.rs         # Comprehensive tests
```

### Core Trait Definition

```rust
// src/effect/sink/trait_def.rs

use std::future::Future;
use crate::effect::Effect;

/// An effect that emits items to a sink during execution.
///
/// SinkEffect provides streaming semantics for output. Unlike WriterEffect
/// which accumulates all writes in memory, SinkEffect streams items to a
/// provided sink function as they occur.
///
/// # When to Use
///
/// - **SinkEffect**: High-volume output, real-time streaming, production logging
/// - **WriterEffect**: Testing, short chains, audit trails needing full history
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
/// use stillwater::effect::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("starting".to_string())
///     .and_then(|_| emit("processing".to_string()))
///     .and_then(|_| emit("done".to_string()))
///     .map(|_| 42);
///
/// // Stream to console
/// let result = effect.run_with_sink(&(), |log| async move {
///     println!("{}", log);
/// }).await;
///
/// assert_eq!(result, Ok(42));
/// # });
/// ```
pub trait SinkEffect: Effect {
    /// The type of items emitted to the sink.
    type Item: Send;

    /// Execute this effect, emitting items to the sink as they occur.
    ///
    /// The sink function is called synchronously for each emitted item,
    /// but the sink itself can be async to support I/O operations.
    ///
    /// # Arguments
    ///
    /// * `env` - The environment for this effect
    /// * `sink` - Function called for each emitted item
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = emit::<_, String, ()>("hello".to_string());
    ///
    /// let result = effect.run_with_sink(&(), |item| async move {
    ///     // Could send to logging service, write to file, etc.
    ///     println!("Received: {}", item);
    /// }).await;
    /// # });
    /// ```
    fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send;
}
```

### Emit Constructor

```rust
// src/effect/sink/emit.rs

use std::marker::PhantomData;
use crate::effect::Effect;
use crate::effect::sink::SinkEffect;

/// An effect that emits a single item to the sink.
///
/// This is the fundamental Sink operation - it emits an item and
/// produces `()` as output.
#[derive(Debug)]
pub struct Emit<T, E, Env> {
    item: T,
    _phantom: PhantomData<fn() -> (E, Env)>,
}

impl<T, E, Env> Clone for Emit<T, E, Env>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for Emit<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = ();
    type Error = E;
    type Env = Env;

    async fn run(self, _env: &Self::Env) -> Result<Self::Output, Self::Error> {
        // When run as a plain Effect, emission is a no-op
        Ok(())
    }
}

impl<T, E, Env> SinkEffect for Emit<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Item = T;

    async fn run_with_sink<S, Fut>(
        self,
        _env: &Self::Env,
        sink: S,
    ) -> Result<(), E>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        sink(self.item).await;
        Ok(())
    }
}

/// Emit a single item to the sink.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let effect = emit::<_, String, ()>("log message".to_string());
///
/// effect.run_with_sink(&(), |log| async move {
///     println!("{}", log);
/// }).await;
/// # });
/// ```
pub fn emit<T, E, Env>(item: T) -> Emit<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    Emit {
        item,
        _phantom: PhantomData,
    }
}
```

### SinkAndThen Combinator

```rust
// src/effect/sink/and_then.rs

use crate::effect::Effect;
use crate::effect::sink::SinkEffect;

/// Chains dependent SinkEffects, streaming from both.
///
/// When the first effect completes successfully, its output is passed
/// to the function to produce the next effect. Items from both effects
/// are streamed to the sink in order.
pub struct SinkAndThen<E, F> {
    pub(crate) inner: E,
    pub(crate) f: F,
}

impl<E, F> std::fmt::Debug for SinkAndThen<E, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SinkAndThen")
            .field("inner", &"<effect>")
            .field("f", &"<function>")
            .finish()
    }
}

impl<E, F, E2> Effect for SinkAndThen<E, F>
where
    E: SinkEffect,
    E2: SinkEffect<Error = E::Error, Env = E::Env, Item = E::Item>,
    F: FnOnce(E::Output) -> E2 + Send,
{
    type Output = E2::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let value = self.inner.run(env).await?;
        (self.f)(value).run(env).await
    }
}

impl<E, F, E2> SinkEffect for SinkAndThen<E, F>
where
    E: SinkEffect,
    E2: SinkEffect<Error = E::Error, Env = E::Env, Item = E::Item>,
    F: FnOnce(E::Output) -> E2 + Send,
{
    type Item = E::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<E2::Output, E::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        // Run first effect, streaming to sink
        let value = self.inner.run_with_sink(env, &sink).await?;

        // Run second effect with same sink
        let next_effect = (self.f)(value);
        next_effect.run_with_sink(env, sink).await
    }
}
```

### Extension Trait with run_collecting

```rust
// src/effect/sink/ext.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::effect::sink::{SinkEffect, BoxedSinkEffect};
use crate::effect::sink::tap_emit::TapEmit;

/// Extension trait providing combinator methods for all SinkEffects.
pub trait SinkEffectExt: SinkEffect {
    /// Emit a derived value after success.
    ///
    /// If this effect succeeds, the function is called with a reference
    /// to the output, and the result is emitted to the sink.
    fn tap_emit<F>(self, f: F) -> TapEmit<Self, F>
    where
        Self: Sized,
        Self::Output: Clone + Send,
        F: FnOnce(&Self::Output) -> Self::Item + Send,
    {
        TapEmit { inner: self, f }
    }

    /// Execute and collect all emissions (for testing).
    ///
    /// This bridges SinkEffect to WriterEffect-like semantics,
    /// collecting all emitted items into a Vec for assertions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::sink::prelude::*;
    ///
    /// # tokio_test::block_on(async {
    /// let effect = emit::<_, String, ()>("a".to_string())
    ///     .and_then(|_| emit("b".to_string()))
    ///     .and_then(|_| emit("c".to_string()))
    ///     .map(|_| 42);
    ///
    /// let (result, collected) = effect.run_collecting(&()).await;
    /// assert_eq!(result, Ok(42));
    /// assert_eq!(collected, vec!["a", "b", "c"]);
    /// # });
    /// ```
    #[allow(async_fn_in_trait)]
    async fn run_collecting(
        self,
        env: &Self::Env,
    ) -> (Result<Self::Output, Self::Error>, Vec<Self::Item>)
    where
        Self: Sized,
        Self::Item: Send + 'static,
    {
        let collected = Arc::new(Mutex::new(Vec::new()));
        let collected_clone = Arc::clone(&collected);

        let result = self.run_with_sink(env, move |item| {
            let collected = Arc::clone(&collected_clone);
            async move {
                collected.lock().await.push(item);
            }
        }).await;

        let items = Arc::try_unwrap(collected)
            .expect("sink should be dropped")
            .into_inner();

        (result, items)
    }

    /// Convert to a boxed SinkEffect for type erasure.
    fn boxed_sink(self) -> BoxedSinkEffect<Self::Output, Self::Error, Self::Env, Self::Item>
    where
        Self: Sized + Send + 'static,
        Self::Output: Send + 'static,
        Self::Error: Send + 'static,
        Self::Env: Clone + Send + Sync + 'static,
        Self::Item: Send + 'static,
    {
        BoxedSinkEffect::new(self)
    }

    /// Execute, discarding all emissions.
    ///
    /// Useful when you only care about the result, not the output.
    #[allow(async_fn_in_trait)]
    async fn run_ignore_emissions(
        self,
        env: &Self::Env,
    ) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
    {
        self.run_with_sink(env, |_| async {}).await
    }
}

// Blanket implementation
impl<E: SinkEffect> SinkEffectExt for E {}
```

### Traverse and Fold Combinators

```rust
// src/effect/sink/combinators.rs

use crate::effect::sink::SinkEffect;

/// Process each item in a collection, streaming output.
///
/// Similar to WriterEffect's traverse_writer, but streams items
/// to the sink instead of accumulating.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::sink::prelude::*;
///
/// # tokio_test::block_on(async {
/// let items = vec![1, 2, 3, 4, 5];
///
/// let effect = traverse_sink(items, |n| {
///     emit::<_, String, ()>(format!("Processing: {}", n))
///         .map(move |_| n * 10)
/// });
///
/// let (result, logs) = effect.run_collecting(&()).await;
/// assert_eq!(result, Ok(vec![10, 20, 30, 40, 50]));
/// assert_eq!(logs.len(), 5);
/// # });
/// ```
pub fn traverse_sink<I, F, Eff>(
    items: I,
    f: F,
) -> impl SinkEffect<Output = Vec<Eff::Output>, Error = Eff::Error, Env = Eff::Env, Item = Eff::Item>
where
    I: IntoIterator,
    I::IntoIter: Send,
    I::Item: Send,
    F: Fn(I::Item) -> Eff + Send + Sync,
    Eff: SinkEffect,
{
    TraverseSink {
        items: items.into_iter().collect::<Vec<_>>(),
        f,
        _phantom: std::marker::PhantomData,
    }
}

pub struct TraverseSink<T, F, Eff> {
    items: Vec<T>,
    f: F,
    _phantom: std::marker::PhantomData<Eff>,
}

impl<T, F, Eff> Effect for TraverseSink<T, F, Eff>
where
    T: Send,
    F: Fn(T) -> Eff + Send + Sync,
    Eff: SinkEffect,
{
    type Output = Vec<Eff::Output>;
    type Error = Eff::Error;
    type Env = Eff::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::with_capacity(self.items.len());
        for item in self.items {
            results.push((self.f)(item).run(env).await?);
        }
        Ok(results)
    }
}

impl<T, F, Eff> SinkEffect for TraverseSink<T, F, Eff>
where
    T: Send,
    F: Fn(T) -> Eff + Send + Sync,
    Eff: SinkEffect,
{
    type Item = Eff::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        let mut results = Vec::with_capacity(self.items.len());
        for item in self.items {
            results.push((self.f)(item).run_with_sink(env, &sink).await?);
        }
        Ok(results)
    }
}

/// Fold a collection with streaming output.
pub fn fold_sink<I, F, Eff, Acc>(
    items: I,
    init: Acc,
    f: F,
) -> impl SinkEffect<Output = Acc, Error = Eff::Error, Env = Eff::Env, Item = Eff::Item>
where
    I: IntoIterator,
    I::IntoIter: Send,
    I::Item: Send,
    Acc: Send,
    F: Fn(Acc, I::Item) -> Eff + Send + Sync,
    Eff: SinkEffect<Output = Acc>,
{
    FoldSink {
        items: items.into_iter().collect::<Vec<_>>(),
        init,
        f,
        _phantom: std::marker::PhantomData,
    }
}

pub struct FoldSink<T, F, Acc, Eff> {
    items: Vec<T>,
    init: Acc,
    f: F,
    _phantom: std::marker::PhantomData<Eff>,
}

impl<T, F, Acc, Eff> Effect for FoldSink<T, F, Acc, Eff>
where
    T: Send,
    Acc: Send,
    F: Fn(Acc, T) -> Eff + Send + Sync,
    Eff: SinkEffect<Output = Acc>,
{
    type Output = Acc;
    type Error = Eff::Error;
    type Env = Eff::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        let mut acc = self.init;
        for item in self.items {
            acc = (self.f)(acc, item).run(env).await?;
        }
        Ok(acc)
    }
}

impl<T, F, Acc, Eff> SinkEffect for FoldSink<T, F, Acc, Eff>
where
    T: Send,
    Acc: Send,
    F: Fn(Acc, T) -> Eff + Send + Sync,
    Eff: SinkEffect<Output = Acc>,
{
    type Item = Eff::Item;

    async fn run_with_sink<S, Fut>(
        self,
        env: &Self::Env,
        sink: S,
    ) -> Result<Self::Output, Self::Error>
    where
        S: Fn(Self::Item) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send,
    {
        let mut acc = self.init;
        for item in self.items {
            acc = (self.f)(acc, item).run_with_sink(env, &sink).await?;
        }
        Ok(acc)
    }
}
```

### Usage Examples

```rust
// Example: Real-time log streaming
use stillwater::effect::sink::prelude::*;
use stillwater::effect::prelude::*;

// ============================================================================
// Example 1: Common Sink Destinations
// ============================================================================

// Write to stdout (simplest case)
effect.run_with_sink(&env, |log| async move {
    println!("[INFO] {}", log);
}).await;

// Append to log file
effect.run_with_sink(&env, |log| async move {
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open("app.log")
        .await
        .unwrap();
    file.write_all(format!("{}\n", log).as_bytes()).await.unwrap();
}).await;

// Send to tracing
effect.run_with_sink(&env, |log| async move {
    tracing::info!("{}", log);
}).await;

// Push to channel for async processing
let (tx, mut rx) = tokio::sync::mpsc::channel(100);
effect.run_with_sink(&env, move |log| {
    let tx = tx.clone();
    async move { tx.send(log).await.ok(); }
}).await;

// Write to stderr for errors
effect.run_with_sink(&env, |log| async move {
    eprintln!("[ERROR] {}", log);
}).await;

// Insert into database audit table
effect.run_with_sink(&env, |event| async move {
    sqlx::query("INSERT INTO audit_log (event, timestamp) VALUES (?, NOW())")
        .bind(&event)
        .execute(&pool)
        .await
        .ok();
}).await;

// ============================================================================
// Example 2: Basic Streaming with File Output
// ============================================================================

async fn basic_streaming() {
    let effect = emit::<_, String, ()>("Starting process".to_string())
        .and_then(|_| into_sink(pure::<_, String, ()>(42)))
        .tap_emit(|n| format!("Got value: {}", n))
        .and_then(|n| emit(format!("Final: {}", n)).map(move |_| n));

    // Stream to log file
    let log_file = std::sync::Arc::new(tokio::sync::Mutex::new(
        tokio::fs::File::create("process.log").await.unwrap()
    ));

    let result = effect.run_with_sink(&(), move |log| {
        let file = log_file.clone();
        async move {
            use tokio::io::AsyncWriteExt;
            let mut f = file.lock().await;
            f.write_all(format!("{}\n", log).as_bytes()).await.ok();
        }
    }).await;

    assert_eq!(result, Ok(42));
}

// ============================================================================
// Example 3: Testing with run_collecting
// ============================================================================

#[tokio::test]
async fn test_logs_emitted() {
    let effect = emit::<_, String, ()>("step 1".to_string())
        .and_then(|_| emit("step 2".to_string()))
        .and_then(|_| emit("step 3".to_string()))
        .map(|_| "done");

    // Testing: collect for assertions
    let (result, logs) = effect.run_collecting(&()).await;

    assert_eq!(result, Ok("done"));
    assert_eq!(logs, vec!["step 1", "step 2", "step 3"]);
}

// ============================================================================
// Example 4: High-volume processing (constant memory)
// ============================================================================

async fn process_million_items() {
    let items: Vec<i32> = (0..1_000_000).collect();

    let effect = traverse_sink(items, |n| {
        emit::<_, String, ()>(format!("Item {}", n))
            .map(move |_| n * 2)
    });

    // Memory stays constant - items streamed immediately
    let result = effect.run_with_sink(&(), |log| async move {
        // Write to file, send to service, etc.
        writeln!(log_file, "{}", log).await;
    }).await;

    // result: Ok(Vec<i32>) with 1M items
    // Memory: O(output) not O(logs + output)
}

// ============================================================================
// Example 5: Dual execution patterns (production vs testing)
// ============================================================================

/// Create a logging effect that works with both patterns
fn log_operation<Env: Clone + Send + Sync>(
    name: &str,
    value: i32,
) -> impl SinkEffect<Output = i32, Error = String, Env = Env, Item = String> {
    emit::<_, String, Env>(format!("Starting: {}", name))
        .and_then(move |_| {
            into_sink(pure::<_, String, Env>(value * 2))
        })
        .tap_emit(move |v| format!("Result: {}", v))
}

// Production
async fn run_production() {
    let effect = log_operation::<()>("multiply", 21);
    effect.run_with_sink(&(), |log| async move {
        production_logger::info!("{}", log);
    }).await;
}

// Testing
#[tokio::test]
async fn run_test() {
    let effect = log_operation::<()>("multiply", 21);
    let (result, logs) = effect.run_collecting(&()).await;

    assert_eq!(result, Ok(42));
    assert_eq!(logs.len(), 2);
    assert!(logs[0].contains("Starting"));
    assert!(logs[1].contains("Result"));
}

// ============================================================================
// Example 6: Error handling preserves prior emissions
// ============================================================================

async fn error_handling() {
    let effect = emit::<_, String, String>("before error".to_string())
        .and_then(|_| emit("about to fail".to_string()))
        .and_then(|_| {
            into_sink(fail::<i32, String, ()>("something broke".to_string()))
        })
        .and_then(|n| emit("never reached".to_string()).map(move |_| n));

    let (result, logs) = effect.run_collecting(&()).await;

    assert!(result.is_err());
    // Logs up to failure point were still emitted
    assert_eq!(logs, vec!["before error", "about to fail"]);
}
```

## Dependencies

- **Prerequisites**: Spec 037 (Writer Effect) - SinkEffect is the streaming counterpart
- **Affected Components**:
  - `src/effect/mod.rs` - add `sink` module export
  - `src/lib.rs` - optionally add to prelude
- **External Dependencies**: None (uses tokio::sync for testing utilities only)

## Testing Strategy

### Unit Tests

- Test `emit` with various item types
- Test `emit_many` with iterators
- Test `into_sink` preserves Effect behavior
- Test all combinators maintain streaming semantics
- Test `run_collecting` collects all emissions in order
- Test error propagation preserves prior emissions
- Test async sinks with delays/backpressure

### Integration Tests

- Test traverse_sink with large collections
- Test fold_sink accumulator patterns
- Test mixing SinkEffect with regular Effect
- Test boxed effects in heterogeneous collections

### Property Tests

- Items emitted in `and_then` chain appear in order
- `run_collecting` returns same items as streamed to sink
- Memory usage stays constant for large traversals (benchmark)

### Benchmark Tests

- Compare memory usage: Writer vs Sink for 1M items
- Measure streaming latency vs accumulation latency
- Test throughput with async sinks

## Documentation Requirements

### Code Documentation

- Doc comments on all public types and methods
- Examples in doc comments showing production and testing patterns
- Explanation of when to use SinkEffect vs WriterEffect

### Example File

Create `examples/sink_streaming.rs` demonstrating:

```rust
//! Sink Effect example demonstrating streaming output patterns.
//!
//! Run with: cargo run --example sink_streaming

use stillwater::effect::prelude::*;
use stillwater::effect::sink::prelude::*;
use tokio::io::AsyncWriteExt;

// Example 1: Stream to stdout
async fn stream_to_stdout() {
    println!("=== Streaming to stdout ===");

    let effect = emit::<_, String, ()>("Step 1: Initialize".into())
        .and_then(|_| emit("Step 2: Process".into()))
        .and_then(|_| emit("Step 3: Complete".into()))
        .map(|_| 42);

    let result = effect.run_with_sink(&(), |log| async move {
        println!("  {}", log);
    }).await;

    println!("Result: {:?}\n", result);
}

// Example 2: Stream to file
async fn stream_to_file() {
    println!("=== Streaming to file ===");

    let file = std::sync::Arc::new(tokio::sync::Mutex::new(
        tokio::fs::File::create("/tmp/sink_demo.log").await.unwrap()
    ));

    let items = vec![1, 2, 3, 4, 5];
    let effect = traverse_sink(items, |n| {
        emit::<_, String, ()>(format!("Processing item: {}", n))
            .map(move |_| n * 10)
    });

    let result = effect.run_with_sink(&(), move |log| {
        let file = file.clone();
        async move {
            let mut f = file.lock().await;
            f.write_all(format!("{}\n", log).as_bytes()).await.ok();
        }
    }).await;

    println!("Results: {:?}", result);
    println!("Logs written to /tmp/sink_demo.log\n");
}

// Example 3: Testing pattern with run_collecting
async fn testing_example() {
    println!("=== Testing with run_collecting ===");

    let effect = emit::<_, String, ()>("audit: user login".into())
        .and_then(|_| emit("audit: access granted".into()))
        .and_then(|_| emit("audit: resource fetched".into()))
        .map(|_| "success");

    let (result, collected) = effect.run_collecting(&()).await;

    println!("Result: {:?}", result);
    println!("Collected {} audit events:", collected.len());
    for event in &collected {
        println!("  - {}", event);
    }
}

// Example 4: High-volume processing (constant memory)
async fn high_volume_example() {
    println!("\n=== High-volume streaming (1000 items) ===");

    let items: Vec<i32> = (1..=1000).collect();
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = count.clone();

    let effect = traverse_sink(items, |n| {
        emit::<_, String, ()>(format!("Item {}", n))
            .map(move |_| n)
    });

    let _result = effect.run_with_sink(&(), move |_log| {
        let c = count_clone.clone();
        async move {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }).await;

    println!("Streamed {} log entries with constant memory",
             count.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::main]
async fn main() {
    stream_to_stdout().await;
    stream_to_file().await;
    testing_example().await;
    high_volume_example().await;

    println!("\n=== Benefits of Sink Effect ===");
    println!("- O(1) memory: items streamed immediately, not accumulated");
    println!("- Real-time output: logs appear as they happen");
    println!("- Flexible sinks: stdout, files, channels, databases");
    println!("- Testable: run_collecting bridges to Writer semantics");
    println!("- Async-ready: sinks can perform I/O without blocking");
}
```

### README Updates

Add a new section to README.md after the Writer Effect section:

```markdown
### Sink Effect - Streaming Output

For high-volume or long-running processes where accumulating logs in memory
isn't practical, use `SinkEffect` to stream output in real-time:

\`\`\`rust
use stillwater::effect::sink::prelude::*;

// Process items, streaming logs immediately
let effect = traverse_sink(items, |item| {
    emit(format!("Processing: {}", item))
        .map(|_| process(item))
});

// Production: stream to file/service
effect.run_with_sink(&env, |log| async move {
    writeln!(log_file, "{}", log).await;
}).await;

// Testing: collect for assertions
let (result, logs) = effect.run_collecting(&env).await;
assert_eq!(logs.len(), items.len());
\`\`\`

| Use Case | Effect Type | Memory |
|----------|-------------|--------|
| Testing, audit trails | `WriterEffect` | O(n) |
| Production streaming | `SinkEffect` | O(1) |

See [examples/sink_streaming.rs](examples/sink_streaming.rs) for more patterns.
```

### docs/COMPARISON.md Updates

Add SinkEffect to the comparison table showing how Stillwater handles streaming vs accumulation compared to other libraries.

### User Documentation

- Add "Sink Effect" section to main documentation
- Tutorial comparing Writer (pure/testing) vs Sink (streaming/production)
- Migration guide for moving from Writer to Sink
- Best practices for choosing between the two:
  - Use Writer when: testing, need `censor`/`listen`, audit trails, short chains
  - Use Sink when: production logging, high volume, real-time visibility, memory constraints

## Implementation Notes

### Memory Efficiency

SinkEffect achieves O(1) memory for emissions by:
- Not storing emitted items
- Calling sink immediately on each emission
- Dropping items after sink returns

### Async Sink Design

The sink function signature `Fn(Item) -> Fut` where `Fut: Future<Output = ()>`:
- Allows sync sinks: `|item| async { println!("{}", item) }`
- Allows async I/O: `|item| async { send_to_network(item).await }`
- Allows backpressure: sink can await slow consumers

### Relationship to WriterEffect

| Aspect | WriterEffect | SinkEffect |
|--------|--------------|------------|
| Memory | O(n) accumulation | O(1) streaming |
| Algebra | Monoid required | No constraints |
| Testing | Returns collected writes | run_collecting helper |
| Introspection | listen, censor | N/A |
| Use case | Audit trails, testing | Production streaming |

### Future Enhancements

1. **Buffered Sink**: Buffer N items before calling sink
2. **Parallel Sink**: Fan-out to multiple sinks
3. **Filtered Sink**: Apply predicate before emitting
4. **Batched Sink**: Accumulate and emit in batches

## Migration and Compatibility

### Backward Compatibility

Purely additive - no breaking changes to existing code.

### Migration Path

1. **Identify high-volume Writer usage**: Find traverse_writer calls on large collections
2. **Switch to SinkEffect**: Replace with traverse_sink
3. **Update tests**: Use run_collecting for assertions
4. **Configure production sink**: Wire up to logging service

```rust
// Before: Writer accumulates in memory
let (result, logs) = traverse_writer(items, process).run_writer(&env).await;
send_logs(logs); // Batch send after completion

// After: Sink streams immediately
let result = traverse_sink(items, process).run_with_sink(&env, |log| async {
    send_log(log).await // Stream in real-time
}).await;
```
