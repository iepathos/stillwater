---
number: 024
title: Zero-Cost Effect Trait with Opt-In Boxing
category: foundation
priority: critical
status: draft
dependencies: []
created: 2025-11-26
---

# Specification 024: Zero-Cost Effect Trait with Opt-In Boxing

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: None

## Context

### Design Decision: Environment Cloning

**Decision**: The environment (`Env`) must implement `Clone`.

**Rationale**: The core tension in this design is between zero-cost abstractions and type erasure. When we box an effect, we lose static lifetime information. The future returned by `BoxedEffect::run` cannot borrow from the environment because `BoxFuture<'static, ...>` requires no borrowed data.

Three options were considered:

| Option | Tradeoff |
|--------|----------|
| `Env: Clone` | Simple API, slight clone overhead, works everywhere |
| `Arc<Env>` parameter | Zero-copy but changes all signatures, less ergonomic |
| Lifetime parameter on `BoxedEffect<'env, ...>` | Complex, lifetime pollution throughout API |

**Why Clone wins**: Real-world environments are typically composed of `Arc`-wrapped resources (`Arc<DbPool>`, `Arc<Config>`, etc.) making `Clone` essentially free. The simplicity benefit outweighs the theoretical overhead.

```rust
// Typical environment - Clone is cheap (just Arc ref counts)
#[derive(Clone)]
struct AppEnv {
    db: Arc<DatabasePool>,
    config: Arc<Config>,
    http: Arc<HttpClient>,
}
```

### The Current Problem

Stillwater's current `Effect` type uses boxing for every combinator:

```rust
pub struct Effect<T, E, Env> {
    run_fn: Box<dyn FnOnce(&Env) -> BoxFuture<'_, Result<T, E>> + Send>,
}

impl<T, E, Env> Effect<T, E, Env> {
    pub fn map<U, F>(self, f: F) -> Effect<U, E, Env> {
        Effect {
            run_fn: Box::new(move |env| ...),  // NEW BOX every time
        }
    }
}
```

This means:
```rust
Effect::pure(42)           // Box #1
    .map(|x| x + 1)        // Box #2
    .and_then(|x| ...)     // Box #3
    .map(|x| x.to_string()) // Box #4
```

**Four heap allocations** for a simple chain.

### Why This Matters for Rust Adoption

Rust's identity is built on **zero-cost abstractions**: "What you don't use, you don't pay for. What you do use, you couldn't hand-code any better."

Performance-oriented Rustaceans will be skeptical of a library that boxes per combinator, even if the cost is negligible for I/O-bound work. The current documentation's claim that this is acceptable for "workflow orchestration" won't satisfy engineers who value the principle.

### The Standard Rust Pattern

The `futures` crate faces the exact same problem and has broad community acceptance:

```rust
// Zero-cost - each combinator returns a concrete type
let future = async { 42 }
    .map(|x| x + 1)           // Returns Map<..., impl FnOnce...>
    .then(|x| async { x * 2 }); // Returns Then<Map<...>, ...>

// Type: Then<Map<Ready<i32>, ...>, ...>
// NO heap allocation!

// When you NEED type erasure, you explicitly opt in:
let boxed: Pin<Box<dyn Future<Output = i32>>> = future.boxed();
```

**This is the model Stillwater should adopt.**

### When Type Erasure (Boxing) Is Needed

There are exactly three cases where boxing becomes necessary:

#### 1. Storing in Collections

`Vec<T>` requires all elements to be the same type:

```rust
let effect_a = pure(1).map(|x| x + 1);
// Type: Map<Pure<i32, E, Env>, impl FnOnce(i32) -> i32>

let effect_b = pure(2).and_then(|x| pure(x * 2));
// Type: AndThen<Pure<i32, E, Env>, impl FnOnce(i32) -> Pure<i32, E, Env>>

// Can't put different types in a Vec!
let effects: Vec<???> = vec![effect_a, effect_b];  // ERROR

// Solution: explicit boxing
let effects: Vec<BoxedEffect<i32, E, Env>> = vec![
    effect_a.boxed(),
    effect_b.boxed(),
];
```

#### 2. Recursive Effects

A type cannot contain itself without indirection:

```rust
fn countdown(n: i32) -> ??? {
    if n <= 0 {
        pure(0)
    } else {
        pure(n).and_then(move |x| countdown(x - 1))
        // Type would be infinitely nested!
    }
}

// Solution: explicit boxing breaks the infinite type
fn countdown(n: i32) -> BoxedEffect<i32, E, Env> {
    if n <= 0 {
        pure(0).boxed()
    } else {
        pure(n).and_then(move |x| countdown(x - 1)).boxed()
    }
}
```

#### 3. Match Arms with Different Effect Types

All match arms must return the same type:

```rust
fn get_user(source: DataSource) -> ??? {
    match source {
        DataSource::Cache => pure(user),        // Type A
        DataSource::Database => fetch_from_db() // Type B  <- ERROR!
    }
}

// Solution: explicit boxing
fn get_user(source: DataSource) -> BoxedEffect<User, E, Env> {
    match source {
        DataSource::Cache => pure(user).boxed(),
        DataSource::Database => fetch_from_db().boxed(),
    }
}
```

## Objective

Redesign Stillwater's Effect system to be **zero-cost by default** with **opt-in boxing** when type erasure is needed, following the established `futures` crate pattern.

### Goals

1. **Zero-cost by default**: Effect chains compile to the same code as hand-written async functions
2. **Explicit boxing**: Users call `.boxed()` when they need type erasure
3. **Familiar pattern**: Follow the `Future`/`Iterator` trait model that Rustaceans know
4. **Full backward compatibility path**: Provide migration helpers from current API
5. **No performance regression**: Boxed effects should perform the same as current implementation

## Requirements

### Functional Requirements

#### FR1: Effect Trait Definition

- **MUST** define an `Effect` trait that represents a computation
- **MUST** have associated types for `Output`, `Error`, and `Env`
- **MUST** provide a `run` method that returns an `impl Future`
- **MUST** allow implementing types to be zero-sized when possible

```rust
pub trait Effect: Sized + Send {
    type Output: Send;
    type Error: Send;
    type Env: Clone + Send + Sync;  // Clone required for boxing

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}
```

#### FR2: Concrete Combinator Types (Zero-Cost)

- **MUST** provide `Pure<T, E, Env>` for pure values
- **MUST** provide `Fail<T, E, Env>` for failure values
- **MUST** provide `Map<Inner, F>` for mapping success values
- **MUST** provide `MapErr<Inner, F>` for mapping error values
- **MUST** provide `AndThen<Inner, F>` for chaining effects
- **MUST** provide `OrElse<Inner, F>` for error recovery
- **MUST** provide `FromFn<F, Env>` for effects from synchronous functions
- **MUST** provide `FromAsync<F, Fut, Env>` for effects from async functions
- **MUST** provide `FromResult<T, E, Env>` for effects from Result values
- **MUST** provide `Ask<E, Env>` for accessing the full environment
- **MUST** provide `Asks<F, U, E, Env>` for querying the environment
- **MUST** provide `Local<Inner, F>` for modified environment execution
- **MUST NOT** allocate heap memory for these types

#### FR3: Extension Trait for Combinators

- **MUST** provide `EffectExt` trait with combinator methods
- **MUST** have `map`, `map_err`, `and_then`, `or_else`, `local` methods
- **MUST** return concrete types (not boxed) from combinators
- **MUST** be automatically implemented for all `Effect` types
- **MUST** allow error type changes via `map_err` for flexible composition

```rust
pub trait EffectExt: Effect {
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Send,
        U: Send;

    fn map_err<E2, F>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E2 + Send,
        E2: Send;

    fn and_then<E2, F>(self, f: F) -> AndThen<Self, F>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        F: FnOnce(Self::Output) -> E2 + Send;

    fn or_else<E2, F>(self, f: F) -> OrElse<Self, F>
    where
        E2: Effect<Output = Self::Output, Env = Self::Env>,
        F: FnOnce(Self::Error) -> E2 + Send;

    fn local<F, Env2>(self, f: F) -> Local<Self, F>
    where
        F: FnOnce(&Env2) -> Self::Env + Send,
        Env2: Clone + Send + Sync;
}
```

**Note on Error Types**: The `and_then` constraint `E2::Error = Self::Error` means error types must match in a chain. Use `map_err` to convert errors before chaining:

```rust
// Convert errors to enable chaining with different error types
fetch_user(id)                           // Error = DbError
    .map_err(AppError::from)             // Error = AppError
    .and_then(|user| send_email(user))   // Error = AppError (via Into)
```

#### FR4: Boxed Effect Type (Opt-In)

- **MUST** provide `BoxedEffect<T, E, Env>` for type-erased effects
- **MUST** provide `.boxed()` method on `EffectExt` to convert to boxed
- **MUST** implement `Effect` trait for `BoxedEffect`
- **MUST** clone the environment into the boxed future (required for `'static`)
- **SHOULD** provide `BoxedLocalEffect` for non-Send effects

```rust
/// Type-erased effect. Clones the environment into the future.
pub struct BoxedEffect<T, E, Env> {
    // Takes owned Env (cloned from reference) to produce 'static future
    run_fn: Box<dyn FnOnce(Env) -> BoxFuture<'static, Result<T, E>> + Send>,
    _phantom: PhantomData<Env>,
}

impl<T, E, Env> Effect for BoxedEffect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        let env_owned = env.clone();  // Clone here to get 'static
        (self.run_fn)(env_owned)
    }
}

impl<E: Effect> EffectExt for E {
    fn boxed(self) -> BoxedEffect<Self::Output, Self::Error, Self::Env>
    where
        Self: 'static,
    {
        BoxedEffect::new(self)
    }
}
```

**Why Clone?** The `BoxFuture<'static, ...>` returned by boxed effects cannot borrow from the environment. By cloning `Env` into the closure, the future becomes `'static` and can be stored, returned from functions, or used in recursive effects. Since environments typically contain `Arc`-wrapped resources, this clone is cheap.

#### FR5: Constructor Functions

- **MUST** provide `pure<T, E, Env>(value: T)` function
- **MUST** provide `fail<T, E, Env>(error: E)` function
- **MUST** provide `from_fn` for creating effects from functions
- **MUST** provide `from_async` for creating effects from async functions
- **MUST** provide `from_result` for creating effects from Results
- **MUST** provide `from_option` for creating effects from Options

#### FR6: Environment Access (Reader Pattern)

- **MUST** provide `ask<E, Env>()` to get the entire environment
- **MUST** provide `asks<U, E, Env, F>(f: F)` to query environment
- **MUST** provide `local<F>(f: F, effect: E)` for environment modification
- **MUST** return concrete types from these functions

#### FR7: Parallel Execution

- **MUST** provide `par_all` for parallel execution with error accumulation
- **MUST** provide `par_try_all` for parallel execution with fail-fast
- **MUST** provide `race` for racing multiple effects
- **MUST** work with homogeneous collections via `BoxedEffect`
- **SHOULD** provide tuple-based variants for heterogeneous parallel execution

```rust
/// Execute effects in parallel, collecting all results or all errors.
/// Requires boxed effects for homogeneous collection.
pub async fn par_all<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<Vec<T>, Vec<E>>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync;

/// Execute effects in parallel, fail-fast on first error.
pub async fn par_try_all<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<Vec<T>, E>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync;

/// Race effects, returning the first to complete.
pub async fn race<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<T, E>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync;

/// Tuple-based parallel execution for heterogeneous effects (via macro).
/// par2!, par3!, par4! macros for 2-4 effects with different output types.
```

**Design Note**: Parallel execution requires type erasure because `Vec<T>` needs homogeneous types. The tuple-based variants (`par2!`, etc.) allow heterogeneous effects without boxing but require macro generation for each arity.

#### FR8: Resource Management (Bracket)

- **MUST** provide `bracket` for acquire/use/release pattern
- **MUST** guarantee release runs even on error or panic
- **MUST** work with the Effect trait
- **SHOULD** integrate with existing resource scope system (Spec 002)

```rust
/// Bracket pattern for safe resource management.
///
/// Acquires a resource, uses it, and guarantees release.
pub fn bracket<Acquire, Use, Release, R, T, E, Env>(
    acquire: Acquire,
    use_resource: Use,
    release: Release,
) -> Bracket<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    Use: FnOnce(R) -> impl Effect<Output = T, Error = E, Env = Env>,
    Release: FnOnce(R) -> impl Effect<Output = (), Error = E, Env = Env>;

/// Bracket combinator type
pub struct Bracket<Acquire, Use, Release> {
    acquire: Acquire,
    use_fn: Use,
    release: Release,
}

impl<Acquire, Use, Release, R, T, E, Env> Effect for Bracket<Acquire, Use, Release>
where
    Acquire: Effect<Output = R, Error = E, Env = Env>,
    // ... bounds
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        async move {
            let resource = self.acquire.run(env).await?;
            let result = (self.use_fn)(resource.clone()).run(env).await;
            // Release runs regardless of use result
            let _ = (self.release)(resource).run(env).await;
            result
        }
    }
}
```

### Non-Functional Requirements

#### NFR1: Zero-Cost Verification

- Combinators MUST NOT allocate heap memory
- Binary size MUST NOT increase for non-boxed usage
- Generated assembly SHOULD be equivalent to hand-written async code

#### NFR2: Ergonomics

- Return type syntax MUST be reasonable: `impl Effect<Output = T, Error = E, Env = Env>`
- Common patterns SHOULD have type aliases
- Error messages MUST be understandable

#### NFR3: Backward Compatibility

- MUST provide `compat` module for migration from current API
- SHOULD provide deprecation warnings for old patterns
- MUST document migration path clearly

## Acceptance Criteria

### Core Trait Implementation

- [ ] **AC1**: `Effect` trait compiles and works with async
- [ ] **AC2**: `Pure<T, E, Env>` has zero runtime overhead
- [ ] **AC3**: `Map<Inner, F>` has zero heap allocation
- [ ] **AC4**: `AndThen<Inner, F>` has zero heap allocation
- [ ] **AC5**: Chaining 10 combinators results in 0 heap allocations

### Boxed Effect

- [ ] **AC6**: `BoxedEffect` implements `Effect` trait
- [ ] **AC7**: `.boxed()` method available on all effects
- [ ] **AC8**: Can store different effects in `Vec<BoxedEffect<T, E, Env>>`
- [ ] **AC9**: Recursive effects work with `.boxed()`
- [ ] **AC10**: Match arms with different effects work with `.boxed()`

### Integration

- [ ] **AC11**: `bracket` works with new Effect trait
- [ ] **AC12**: `par_all` works with new Effect trait
- [ ] **AC13**: Reader pattern (`ask`, `asks`, `local`) works
- [ ] **AC14**: All existing examples compile with new API

### Performance

- [ ] **AC15**: Benchmark shows zero-cost for non-boxed effects
- [ ] **AC16**: Boxed effects perform same as current implementation
- [ ] **AC17**: No binary size regression for simple programs

## Technical Details

### Implementation Approach

#### Phase 1: Core Trait and Types

```rust
// src/effect/trait.rs

use std::future::Future;

/// The core Effect trait - represents a computation that may perform effects.
///
/// This trait follows the same pattern as `Future` and `Iterator`:
/// - Combinators return concrete types (zero-cost)
/// - Use `.boxed()` when you need type erasure
///
/// **Note**: `Env` requires `Clone` to enable boxing. This is typically cheap
/// when environments contain `Arc`-wrapped resources.
///
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// fn fetch_user(id: i32) -> impl Effect<Output = User, Error = DbError, Env = AppEnv> {
///     asks(|env: &AppEnv| env.db.clone())
///         .and_then(move |db| from_async(move |_| db.fetch_user(id)))
/// }
/// ```
pub trait Effect: Sized + Send {
    /// The success type produced by this effect
    type Output: Send;

    /// The error type that may be produced
    type Error: Send;

    /// The environment type required to run this effect.
    /// Must be Clone to support boxing (cloning is deferred until boxing).
    type Env: Clone + Send + Sync;

    /// Execute this effect with the given environment.
    ///
    /// This is the core method that runs the effect. Most users should use
    /// the `execute` helper method from `EffectExt` instead.
    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}
```

#### Phase 2: Concrete Types

```rust
// src/effect/pure.rs

use std::marker::PhantomData;

/// A pure value wrapped as an Effect.
///
/// This is zero-cost - no heap allocation occurs.
#[derive(Debug, Clone)]
pub struct Pure<T, E, Env> {
    value: T,
    _phantom: PhantomData<(E, Env)>,
}

impl<T, E, Env> Pure<T, E, Env> {
    pub fn new(value: T) -> Self {
        Pure {
            value,
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for Pure<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, _env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        async move { Ok(self.value) }
    }
}

// src/effect/fail.rs

/// A failure value wrapped as an Effect.
#[derive(Debug, Clone)]
pub struct Fail<T, E, Env> {
    error: E,
    _phantom: PhantomData<(T, Env)>,
}

impl<T, E, Env> Effect for Fail<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, _env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        async move { Err(self.error) }
    }
}

// src/effect/map.rs

/// Map combinator - transforms the success value.
///
/// Zero-cost: no heap allocation.
pub struct Map<Inner, F> {
    inner: Inner,
    f: F,
}

impl<Inner, F, U> Effect for Map<Inner, F>
where
    Inner: Effect,
    F: FnOnce(Inner::Output) -> U + Send,
    U: Send,
{
    type Output = U;
    type Error = Inner::Error;
    type Env = Inner::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<U, Self::Error>> + Send {
        async move {
            let value = self.inner.run(env).await?;
            Ok((self.f)(value))
        }
    }
}

// src/effect/and_then.rs

/// AndThen combinator - chains dependent effects.
///
/// Zero-cost: no heap allocation.
pub struct AndThen<Inner, F> {
    inner: Inner,
    f: F,
}

impl<Inner, F, E2> Effect for AndThen<Inner, F>
where
    Inner: Effect,
    E2: Effect<Error = Inner::Error, Env = Inner::Env>,
    F: FnOnce(Inner::Output) -> E2 + Send,
{
    type Output = E2::Output;
    type Error = Inner::Error;
    type Env = Inner::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let value = self.inner.run(env).await?;
            (self.f)(value).run(env).await
        }
    }
}

// src/effect/map_err.rs

/// MapErr combinator - transforms the error value.
///
/// Zero-cost: no heap allocation.
pub struct MapErr<Inner, F> {
    inner: Inner,
    f: F,
}

impl<Inner, F, E2> Effect for MapErr<Inner, F>
where
    Inner: Effect,
    F: FnOnce(Inner::Error) -> E2 + Send,
    E2: Send,
{
    type Output = Inner::Output;
    type Error = E2;
    type Env = Inner::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, E2>> + Send {
        async move {
            self.inner.run(env).await.map_err(self.f)
        }
    }
}

// src/effect/or_else.rs

/// OrElse combinator - recovers from errors.
///
/// Zero-cost: no heap allocation.
pub struct OrElse<Inner, F> {
    inner: Inner,
    f: F,
}

impl<Inner, F, E2> Effect for OrElse<Inner, F>
where
    Inner: Effect,
    E2: Effect<Output = Inner::Output, Env = Inner::Env>,
    F: FnOnce(Inner::Error) -> E2 + Send,
{
    type Output = Inner::Output;
    type Error = E2::Error;
    type Env = Inner::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.inner.run(env).await {
                Ok(value) => Ok(value),
                Err(e) => (self.f)(e).run(env).await,
            }
        }
    }
}

// src/effect/from_fn.rs

/// Effect from a synchronous function.
///
/// Zero-cost: no heap allocation.
pub struct FromFn<F, Env> {
    f: F,
    _phantom: PhantomData<Env>,
}

impl<F, Env> FromFn<F, Env> {
    pub fn new(f: F) -> Self {
        FromFn { f, _phantom: PhantomData }
    }
}

impl<F, T, E, Env> Effect for FromFn<F, Env>
where
    F: FnOnce(&Env) -> Result<T, E> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        async move { (self.f)(env) }
    }
}

// src/effect/from_async.rs

/// Effect from an async function.
///
/// Zero-cost: no heap allocation (beyond the future itself).
pub struct FromAsync<F, Env> {
    f: F,
    _phantom: PhantomData<Env>,
}

impl<F, Env> FromAsync<F, Env> {
    pub fn new(f: F) -> Self {
        FromAsync { f, _phantom: PhantomData }
    }
}

impl<F, Fut, T, E, Env> Effect for FromAsync<F, Env>
where
    F: FnOnce(&Env) -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        (self.f)(env)
    }
}

// src/effect/from_result.rs

/// Effect from a Result value.
///
/// Zero-cost: no heap allocation.
pub struct FromResult<T, E, Env> {
    result: Result<T, E>,
    _phantom: PhantomData<Env>,
}

impl<T, E, Env> FromResult<T, E, Env> {
    pub fn new(result: Result<T, E>) -> Self {
        FromResult { result, _phantom: PhantomData }
    }
}

impl<T, E, Env> Effect for FromResult<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, _env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        async move { self.result }
    }
}

// src/effect/reader.rs

/// Get the entire environment (cloned).
///
/// Zero-cost struct, but clones Env at runtime.
pub struct Ask<E, Env> {
    _phantom: PhantomData<(E, Env)>,
}

impl<E, Env> Ask<E, Env> {
    pub fn new() -> Self {
        Ask { _phantom: PhantomData }
    }
}

impl<E, Env> Default for Ask<E, Env> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, Env> Effect for Ask<E, Env>
where
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = Env;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<Env, E>> + Send {
        let env_clone = env.clone();
        async move { Ok(env_clone) }
    }
}

/// Query a value from the environment.
///
/// Zero-cost: no heap allocation.
pub struct Asks<F, E, Env> {
    f: F,
    _phantom: PhantomData<(E, Env)>,
}

impl<F, E, Env> Asks<F, E, Env> {
    pub fn new(f: F) -> Self {
        Asks { f, _phantom: PhantomData }
    }
}

impl<F, U, E, Env> Effect for Asks<F, E, Env>
where
    F: FnOnce(&Env) -> U + Send,
    U: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = U;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<U, E>> + Send {
        async move { Ok((self.f)(env)) }
    }
}

/// Run an effect with a modified environment.
///
/// Zero-cost: no heap allocation.
pub struct Local<Inner, F> {
    inner: Inner,
    f: F,
}

impl<Inner, F> Local<Inner, F> {
    pub fn new(inner: Inner, f: F) -> Self {
        Local { inner, f }
    }
}

impl<Inner, F, Env2> Effect for Local<Inner, F>
where
    Inner: Effect,
    F: FnOnce(&Env2) -> Inner::Env + Send,
    Env2: Clone + Send + Sync,
{
    type Output = Inner::Output;
    type Error = Inner::Error;
    type Env = Env2;

    fn run(self, env: &Env2) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        let inner_env = (self.f)(env);
        async move { self.inner.run(&inner_env).await }
    }
}
```

#### Phase 3: Extension Trait

```rust
// src/effect/ext.rs

/// Extension trait providing combinator methods for all Effects.
///
/// This trait is automatically implemented for all types that implement `Effect`.
pub trait EffectExt: Effect {
    /// Transform the success value.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = pure::<_, String, ()>(42).map(|x| x * 2);
    /// ```
    fn map<U, F>(self, f: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> U + Send,
        U: Send,
    {
        Map { inner: self, f }
    }

    /// Transform the error value.
    fn map_err<E2, F>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E2 + Send,
        E2: Send,
    {
        MapErr { inner: self, f }
    }

    /// Chain a dependent effect.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = pure::<_, String, ()>(42)
    ///     .and_then(|x| pure(x * 2));
    /// ```
    fn and_then<E2, F>(self, f: F) -> AndThen<Self, F>
    where
        E2: Effect<Error = Self::Error, Env = Self::Env>,
        F: FnOnce(Self::Output) -> E2 + Send,
    {
        AndThen { inner: self, f }
    }

    /// Recover from an error.
    fn or_else<E2, F>(self, f: F) -> OrElse<Self, F>
    where
        E2: Effect<Output = Self::Output, Env = Self::Env>,
        F: FnOnce(Self::Error) -> E2 + Send,
    {
        OrElse { inner: self, f }
    }

    /// Convert to a boxed effect for type erasure.
    ///
    /// Use this when you need to:
    /// - Store effects in collections
    /// - Return different effect types from match arms
    /// - Create recursive effects
    ///
    /// # Example
    ///
    /// ```rust
    /// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
    ///     pure(1).boxed(),
    ///     pure(2).map(|x| x * 2).boxed(),
    /// ];
    /// ```
    fn boxed(self) -> BoxedEffect<Self::Output, Self::Error, Self::Env>
    where
        Self: 'static,
    {
        BoxedEffect::new(self)
    }

    /// Run and await the effect.
    ///
    /// Convenience method combining run + await.
    async fn execute(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        self.run(env).await
    }
}

// Blanket implementation
impl<E: Effect> EffectExt for E {}
```

#### Phase 4: Boxed Effect

```rust
// src/effect/boxed.rs

use futures::future::BoxFuture;
use std::marker::PhantomData;

/// A type-erased effect.
///
/// Use `BoxedEffect` when you need to:
/// - Store different effect types in a collection
/// - Return different effects from match arms
/// - Create recursive effect functions
///
/// **Note**: Boxing clones the environment to achieve `'static` lifetime.
/// This is cheap when `Env` contains `Arc`-wrapped resources.
///
/// # Example
///
/// ```rust
/// // Store different effects in a Vec
/// let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
///     pure(1).boxed(),
///     pure(2).and_then(|x| pure(x * 2)).boxed(),
/// ];
///
/// // Recursive effect
/// fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
///     if n <= 0 {
///         pure(0).boxed()
///     } else {
///         pure(n)
///             .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
///             .boxed()
///     }
/// }
/// ```
pub struct BoxedEffect<T, E, Env> {
    // Takes OWNED Env (cloned from reference at run time)
    run_fn: Box<dyn FnOnce(Env) -> BoxFuture<'static, Result<T, E>> + Send>,
    _phantom: PhantomData<Env>,
}

impl<T, E, Env> BoxedEffect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    /// Create a boxed effect from any effect.
    ///
    /// The environment will be cloned when the effect is run.
    pub fn new<Eff>(effect: Eff) -> Self
    where
        Eff: Effect<Output = T, Error = E, Env = Env> + 'static,
    {
        BoxedEffect {
            run_fn: Box::new(move |env: Env| {
                // env is now owned, so the async block is 'static
                Box::pin(async move { effect.run(&env).await })
            }),
            _phantom: PhantomData,
        }
    }
}

impl<T, E, Env> Effect for BoxedEffect<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Clone + Send + Sync,
{
    type Output = T;
    type Error = E;
    type Env = Env;

    fn run(self, env: &Env) -> impl Future<Output = Result<T, E>> + Send {
        let env_owned = env.clone();  // Clone here for 'static lifetime
        (self.run_fn)(env_owned)
    }
}
```

#### Phase 5: Constructor Functions

```rust
// src/effect/constructors.rs

/// Create a pure effect that succeeds with the given value.
///
/// Zero-cost: no heap allocation.
///
/// # Example
///
/// ```rust
/// let effect = pure::<_, String, ()>(42);
/// assert_eq!(effect.execute(&()).await, Ok(42));
/// ```
pub fn pure<T, E, Env>(value: T) -> Pure<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    Pure::new(value)
}

/// Create an effect that fails with the given error.
///
/// Zero-cost: no heap allocation.
pub fn fail<T, E, Env>(error: E) -> Fail<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    Fail::new(error)
}

/// Create an effect from a function.
pub fn from_fn<T, E, Env, F>(f: F) -> FromFn<F, Env>
where
    F: FnOnce(&Env) -> Result<T, E> + Send,
    T: Send,
    E: Send,
    Env: Sync,
{
    FromFn::new(f)
}

/// Create an effect from an async function.
pub fn from_async<T, E, Env, F, Fut>(f: F) -> FromAsync<F, Env>
where
    F: FnOnce(&Env) -> Fut + Send,
    Fut: Future<Output = Result<T, E>> + Send,
    T: Send,
    E: Send,
    Env: Sync,
{
    FromAsync::new(f)
}

/// Create an effect from a Result.
pub fn from_result<T, E, Env>(result: Result<T, E>) -> FromResult<T, E, Env>
where
    T: Send,
    E: Send,
    Env: Sync,
{
    FromResult::new(result)
}

/// Get the entire environment.
pub fn ask<E, Env>() -> Ask<E, Env>
where
    Env: Clone + Send + Sync,
    E: Send,
{
    Ask::new()
}

/// Query a value from the environment.
pub fn asks<U, E, Env, F>(f: F) -> Asks<F, E, Env>
where
    F: FnOnce(&Env) -> U + Send,
    U: Send,
    E: Send,
    Env: Sync,
{
    Asks::new(f)
}

/// Run an effect with a modified environment.
pub fn local<Inner, F, Env2>(f: F, inner: Inner) -> Local<Inner, F>
where
    Inner: Effect,
    F: FnOnce(&Env2) -> Inner::Env + Send,
    Env2: Clone + Send + Sync,
{
    Local::new(inner, f)
}
```

#### Phase 6: Parallel Execution

```rust
// src/effect/parallel.rs

use futures::future::{join_all, select_all};

/// Execute boxed effects in parallel, collecting all results or all errors.
///
/// Returns `Ok(results)` if all effects succeed, `Err(errors)` if any fail.
/// All effects run to completion regardless of individual failures.
pub async fn par_all<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<Vec<T>, Vec<E>>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    let futures: Vec<_> = effects
        .into_iter()
        .map(|eff| eff.run(env))
        .collect();

    let results: Vec<Result<T, E>> = join_all(futures).await;

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for result in results {
        match result {
            Ok(value) => successes.push(value),
            Err(e) => failures.push(e),
        }
    }

    if failures.is_empty() {
        Ok(successes)
    } else {
        Err(failures)
    }
}

/// Execute boxed effects in parallel, fail-fast on first error.
///
/// Returns `Ok(results)` if all succeed, `Err(first_error)` on first failure.
/// Note: Other effects may continue running after the first error.
pub async fn par_try_all<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<Vec<T>, E>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    let futures: Vec<_> = effects
        .into_iter()
        .map(|eff| eff.run(env))
        .collect();

    let results: Vec<Result<T, E>> = join_all(futures).await;

    results.into_iter().collect()
}

/// Race effects, returning the first to complete successfully.
///
/// Returns the result of the first effect to complete.
/// Other effects are dropped (cancelled).
pub async fn race<T, E, Env>(
    effects: Vec<BoxedEffect<T, E, Env>>,
    env: &Env,
) -> Result<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Clone + Send + Sync + 'static,
{
    if effects.is_empty() {
        panic!("race called with empty effects vec");
    }

    let futures: Vec<_> = effects
        .into_iter()
        .map(|eff| Box::pin(eff.run(env)))
        .collect();

    let (result, _index, _remaining) = select_all(futures).await;
    result
}

/// Execute two effects in parallel (heterogeneous).
///
/// Zero-cost when effects have concrete types.
pub async fn par2<E1, E2>(
    e1: E1,
    e2: E2,
    env: &E1::Env,
) -> (Result<E1::Output, E1::Error>, Result<E2::Output, E2::Error>)
where
    E1: Effect,
    E2: Effect<Env = E1::Env>,
{
    futures::join!(e1.run(env), e2.run(env))
}

/// Execute three effects in parallel (heterogeneous).
pub async fn par3<E1, E2, E3>(
    e1: E1,
    e2: E2,
    e3: E3,
    env: &E1::Env,
) -> (
    Result<E1::Output, E1::Error>,
    Result<E2::Output, E2::Error>,
    Result<E3::Output, E3::Error>,
)
where
    E1: Effect,
    E2: Effect<Env = E1::Env>,
    E3: Effect<Env = E1::Env>,
{
    futures::join!(e1.run(env), e2.run(env), e3.run(env))
}

// Macro for arbitrary parallel execution with tuple return
#[macro_export]
macro_rules! par {
    ($env:expr; $($effect:expr),+ $(,)?) => {
        futures::join!($($effect.run($env)),+)
    };
}
```

### Module Structure

```
src/
├── lib.rs
├── effect/
│   ├── mod.rs              # Module root, re-exports
│   ├── trait.rs            # Effect trait definition
│   ├── ext.rs              # EffectExt extension trait
│   ├── boxed.rs            # BoxedEffect type
│   ├── constructors.rs     # pure, fail, from_fn, etc.
│   ├── combinators/
│   │   ├── mod.rs
│   │   ├── pure.rs         # Pure<T, E, Env>
│   │   ├── fail.rs         # Fail<T, E, Env>
│   │   ├── map.rs          # Map<Inner, F>
│   │   ├── map_err.rs      # MapErr<Inner, F>
│   │   ├── and_then.rs     # AndThen<Inner, F>
│   │   ├── or_else.rs      # OrElse<Inner, F>
│   │   ├── from_fn.rs      # FromFn<F, Env>
│   │   ├── from_async.rs   # FromAsync<F, Env>
│   │   └── from_result.rs  # FromResult<T, E, Env>
│   ├── reader.rs           # Ask, Asks, Local, ask(), asks(), local()
│   ├── parallel.rs         # par_all, par_try_all, race, par2, par3, par!
│   ├── bracket.rs          # Bracket, bracket()
│   └── prelude.rs          # Common imports
├── compat/                 # Backward compatibility
│   ├── mod.rs
│   └── legacy.rs           # Old Effect struct as type alias
└── ...
```

### Prelude

```rust
// src/effect/prelude.rs

pub use crate::effect::{
    // Traits
    Effect,
    EffectExt,

    // Combinator Types (for advanced use, usually impl Effect suffices)
    BoxedEffect,
    Pure,
    Fail,
    Map,
    MapErr,
    AndThen,
    OrElse,
    FromFn,
    FromAsync,
    FromResult,
    Ask,
    Asks,
    Local,
    Bracket,

    // Constructors
    pure,
    fail,
    from_fn,
    from_async,
    from_result,
    ask,
    asks,
    local,
    bracket,

    // Parallel (homogeneous, requires boxing)
    par_all,
    par_try_all,
    race,

    // Parallel (heterogeneous, zero-cost)
    par2,
    par3,
};

// Re-export the par! macro
pub use crate::par;
```

## Dependencies

### Prerequisites
- None (this is a foundational change)

### Affected Components
- All existing Effect-based code
- Resource scopes (Spec 002)
- Tracing integration (Spec 023)
- All examples and documentation

### External Dependencies
- `futures` crate (for `BoxFuture`)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pure_returns_value() {
        let effect = pure::<_, String, ()>(42);
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn fail_returns_error() {
        let effect = fail::<i32, _, ()>("error".to_string());
        assert_eq!(effect.execute(&()).await, Err("error".to_string()));
    }

    #[tokio::test]
    async fn map_transforms_value() {
        let effect = pure::<_, String, ()>(21).map(|x| x * 2);
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn and_then_chains_effects() {
        let effect = pure::<_, String, ()>(21)
            .and_then(|x| pure(x * 2));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn boxed_allows_collection_storage() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            pure(2).map(|x| x * 2).boxed(),
            pure(3).and_then(|x| pure(x * 3)).boxed(),
        ];

        let mut results = Vec::new();
        for effect in effects {
            results.push(effect.execute(&()).await.unwrap());
        }
        assert_eq!(results, vec![1, 4, 9]);
    }

    #[tokio::test]
    async fn boxed_allows_recursion() {
        fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
            if n <= 0 {
                pure(0).boxed()
            } else {
                pure(n)
                    .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
                    .boxed()
            }
        }

        assert_eq!(countdown(5).execute(&()).await, Ok(15)); // 5+4+3+2+1+0
    }

    #[tokio::test]
    async fn boxed_allows_match_arms() {
        fn get_value(use_double: bool) -> BoxedEffect<i32, String, ()> {
            match use_double {
                true => pure(21).map(|x| x * 2).boxed(),
                false => pure(42).boxed(),
            }
        }

        assert_eq!(get_value(true).execute(&()).await, Ok(42));
        assert_eq!(get_value(false).execute(&()).await, Ok(42));
    }

    // === New tests for added functionality ===

    #[tokio::test]
    async fn map_err_transforms_error() {
        let effect = fail::<i32, _, ()>("error")
            .map_err(|e: &str| format!("wrapped: {}", e));
        assert_eq!(effect.execute(&()).await, Err("wrapped: error".to_string()));
    }

    #[tokio::test]
    async fn map_err_preserves_success() {
        let effect = pure::<_, &str, ()>(42)
            .map_err(|e| format!("wrapped: {}", e));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn or_else_recovers_from_error() {
        let effect = fail::<i32, _, ()>("error")
            .or_else(|_| pure(42));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn or_else_preserves_success() {
        let effect = pure::<_, String, ()>(42)
            .or_else(|_| pure(0));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn from_fn_accesses_environment() {
        #[derive(Clone)]
        struct Env { value: i32 }

        let effect = from_fn::<_, String, _, _>(|env: &Env| Ok(env.value * 2));
        assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
    }

    #[tokio::test]
    async fn from_async_works() {
        let effect = from_async::<_, String, (), _, _>(|_| async { Ok(42) });
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn from_result_ok() {
        let effect = from_result::<_, String, ()>(Ok(42));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn from_result_err() {
        let effect = from_result::<i32, _, ()>(Err("error".to_string()));
        assert_eq!(effect.execute(&()).await, Err("error".to_string()));
    }

    #[tokio::test]
    async fn ask_clones_environment() {
        #[derive(Clone, PartialEq, Debug)]
        struct Env { value: i32 }

        let effect = ask::<String, Env>();
        assert_eq!(effect.execute(&Env { value: 42 }).await, Ok(Env { value: 42 }));
    }

    #[tokio::test]
    async fn asks_queries_environment() {
        #[derive(Clone)]
        struct Env { value: i32 }

        let effect = asks::<_, String, _, _>(|env: &Env| env.value * 2);
        assert_eq!(effect.execute(&Env { value: 21 }).await, Ok(42));
    }

    #[tokio::test]
    async fn local_modifies_environment() {
        #[derive(Clone)]
        struct OuterEnv { multiplier: i32 }
        #[derive(Clone)]
        struct InnerEnv { value: i32 }

        let inner_effect = asks::<_, String, InnerEnv, _>(|env| env.value);
        let effect = local(
            |outer: &OuterEnv| InnerEnv { value: 21 * outer.multiplier },
            inner_effect,
        );

        assert_eq!(effect.execute(&OuterEnv { multiplier: 2 }).await, Ok(42));
    }

    #[tokio::test]
    async fn par_all_collects_successes() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            pure(2).boxed(),
            pure(3).boxed(),
        ];

        let result = par_all(effects, &()).await;
        assert_eq!(result, Ok(vec![1, 2, 3]));
    }

    #[tokio::test]
    async fn par_all_collects_errors() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            fail("error1".to_string()).boxed(),
            fail("error2".to_string()).boxed(),
        ];

        let result = par_all(effects, &()).await;
        assert_eq!(result, Err(vec!["error1".to_string(), "error2".to_string()]));
    }

    #[tokio::test]
    async fn par_try_all_succeeds() {
        let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
            pure(1).boxed(),
            pure(2).boxed(),
        ];

        let result = par_try_all(effects, &()).await;
        assert_eq!(result, Ok(vec![1, 2]));
    }

    #[tokio::test]
    async fn par2_runs_heterogeneous_effects() {
        let e1 = pure::<_, String, ()>(42);
        let e2 = pure::<_, String, ()>("hello".to_string());

        let (r1, r2) = par2(e1, e2, &()).await;
        assert_eq!(r1, Ok(42));
        assert_eq!(r2, Ok("hello".to_string()));
    }

    #[tokio::test]
    async fn error_type_conversion_chain() {
        // Demonstrates the pattern for chaining effects with different error types
        #[derive(Debug, PartialEq)]
        enum AppError { Db(String), Network(String) }

        let effect = pure::<_, String, ()>(42)
            .map_err(AppError::Db)
            .and_then(|x| pure::<_, String, ()>(x * 2).map_err(AppError::Network));

        assert_eq!(effect.execute(&()).await, Ok(84));
    }
}
```

### Zero-Cost Verification Tests

```rust
#[test]
fn pure_is_zero_sized_ignoring_value() {
    // Pure only stores the value, no extra overhead
    assert_eq!(
        std::mem::size_of::<Pure<i32, String, ()>>(),
        std::mem::size_of::<i32>() + std::mem::size_of::<PhantomData<(String, ())>>()
    );
}

#[test]
fn map_only_stores_inner_and_function() {
    // Map stores inner effect + function, no Box
    let effect = pure::<i32, String, ()>(42);
    let mapped = effect.map(|x| x + 1);

    // Size should be inner + closure, not include any Box
    // (exact size depends on closure, but no heap allocation)
}
```

### Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_effect_chain(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("zero_cost_chain_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1)
                    .map(|x| x + 1);
                black_box(effect.execute(&()).await)
            })
        })
    });

    c.bench_function("boxed_chain_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let effect = pure::<_, String, ()>(1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed()
                    .map(|x| x + 1).boxed();
                black_box(effect.execute(&()).await)
            })
        })
    });
}
```

## Documentation Requirements

### Code Documentation
- Full rustdoc for `Effect` trait with examples
- Full rustdoc for all combinator types
- Full rustdoc for `BoxedEffect` with when-to-use guidance
- Module-level documentation explaining the design

### User Documentation
- Update README with new API
- Update PHILOSOPHY.md with accurate zero-cost claims
- Create migration guide from old API
- Update all examples

### Architecture Updates
- Update DESIGN.md with new Effect architecture
- Update ARCHITECTURE.md if it exists

## Migration and Compatibility

### Breaking Changes

This is a significant API change:

| Old API | New API |
|---------|---------|
| `Effect<T, E, Env>` struct | `impl Effect<Output=T, Error=E, Env=Env>` |
| `Effect::pure(x)` | `pure::<_, E, Env>(x)` |
| `Effect::fail(e)` | `fail::<T, _, Env>(e)` |
| `.run(&env).await` | `.execute(&env).await` or `.run(&env).await` |

### Migration Path

```rust
// Old code
fn old_workflow() -> Effect<User, AppError, AppEnv> {
    Effect::pure(user_id)
        .and_then(|id| fetch_user(id))
        .map(|user| user.name)
}

// New code - zero cost
fn new_workflow() -> impl Effect<Output = String, Error = AppError, Env = AppEnv> {
    pure(user_id)
        .and_then(|id| fetch_user(id))
        .map(|user| user.name)
}

// New code - when you need concrete type
fn new_workflow_boxed() -> BoxedEffect<String, AppError, AppEnv> {
    pure(user_id)
        .and_then(|id| fetch_user(id))
        .map(|user| user.name)
        .boxed()
}
```

### Compatibility Module

```rust
// src/compat/legacy.rs

/// Type alias for backward compatibility.
///
/// This provides the old `Effect<T, E, Env>` API as a type alias to `BoxedEffect`.
/// Use this during migration, then update to the new zero-cost API.
#[deprecated(
    since = "0.8.0",
    note = "Use `impl Effect<...>` for zero-cost or `BoxedEffect` for type erasure"
)]
pub type Effect<T, E, Env> = BoxedEffect<T, E, Env>;

/// Deprecated: Use `pure()` function instead.
#[deprecated(since = "0.8.0", note = "Use `pure()` function instead")]
impl<T, E, Env> BoxedEffect<T, E, Env>
where
    T: Send + 'static,
    E: Send + 'static,
    Env: Sync + 'static,
{
    pub fn pure(value: T) -> Self {
        crate::effect::pure(value).boxed()
    }

    pub fn fail(error: E) -> Self {
        crate::effect::fail(error).boxed()
    }
}
```

## Implementation Notes

### Why This Design?

| Choice | Rationale |
|--------|-----------|
| Trait-based | Follows `Future`/`Iterator` pattern, familiar to Rustaceans |
| Concrete combinator types | Enables zero-cost - compiler can inline everything |
| Explicit `.boxed()` | User controls when allocation happens |
| Extension trait | Cleaner API than methods on each type |
| Constructor functions | Type inference works better than associated functions |

### Potential Challenges

1. **Type inference**: May need explicit type annotations in some cases
2. **Error messages**: Deeply nested types can produce confusing errors
3. **IDE support**: Some IDEs struggle with `impl Trait` return types
4. **Documentation**: Need clear guidance on when to use `.boxed()`

### Future Enhancements

1. **Type aliases**: `type AppEffect<T> = impl Effect<Output = T, Error = AppError, Env = AppEnv>`
2. **Macro sugar**: `effect! { ... }` for building effect chains
3. **More combinators**: `zip`, `flatten`, `filter`, etc.
4. **LocalBoxedEffect**: For non-Send effects

## Success Metrics

### Quantitative
- Zero heap allocations for non-boxed effect chains
- No binary size increase for simple programs
- Benchmark shows zero-cost claims are accurate

### Qualitative
- Positive reception from Rust community
- Clear migration path for existing users
- Documentation accurately reflects behavior

---

*"Zero-cost by default, boxing when you need it."*
