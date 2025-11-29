---
number: 040
title: Effect recover Combinator for Partial Error Recovery
category: foundation
priority: high
status: implementation-ready
dependencies: ["Predicate trait (src/predicate/mod.rs)"]
created: 2025-11-27
updated: 2025-11-28
---

# Specification 040: Effect recover Combinator for Partial Error Recovery

**Category**: foundation
**Priority**: high
**Status**: implementation-ready
**Dependencies**: Predicate trait (`src/predicate/mod.rs`)

---

## Summary

This spec adds selective error recovery combinators to the Effect system, filling the gap between total recovery (`or_else`) and no recovery. The implementation leverages stillwater's existing `Predicate<Error>` trait to enable composable, reusable error matching.

**Key Features**:
- Five new combinators: `recover`, `recover_with`, `recover_some`, `fallback`, `fallback_to`
- Predicate-based error matching with composition support (`.and()`, `.or()`, `.not()`)
- Zero-cost abstraction with concrete types
- Backward compatible (closures work via blanket impl)

**Example**:
```rust
use stillwater::effect::prelude::*;
use stillwater::predicate::PredicateExt;

// Define composable error predicates
let is_transient = |e: &Error| matches!(e, Error::Timeout | Error::NetworkError);
let is_client_error = |e: &Error| matches!(e, Error::NotFound);

// Selective recovery with composition
fetch_data()
    .recover(is_transient, |_| retry())
    .recover(is_client_error, |_| pure(default()))
    // Other errors propagate
```

---

## Context

### The Problem

Error handling in Effect chains currently supports only two patterns:
1. **Total recovery** via `or_else` - handles ALL errors
2. **No recovery** - let errors propagate

There's no way to handle SOME errors while letting others propagate:

```rust
// Current: Must handle ALL errors or NONE
let effect = fetch_from_cache(id)
    .or_else(|e| {
        // Forced to handle ALL errors here
        // What if we only want to recover from CacheMiss?
        fetch_from_db(id)
    });

// What we want: Selective recovery
let effect = fetch_from_cache(id)
    .recover(
        |e| matches!(e, Error::CacheMiss),  // Only recover from CacheMiss
        |_| fetch_from_db(id)                // Fallback for CacheMiss only
    );
    // Other errors (e.g., NetworkError) still propagate!
```

### Common Patterns This Enables

1. **Cache fallback**: Recover from cache miss, propagate other errors
2. **Retry on specific errors**: Only retry transient failures
3. **Graceful degradation**: Provide defaults for non-critical failures
4. **Circuit breaker integration**: Different handling for different failure modes

### Prior Art

- **Scala ZIO**: `catchSome`, `catchAll`, `orElse`
- **Haskell**: `catch`, `catchJust`
- **Java CompletableFuture**: `exceptionally`, `handle`
- **JavaScript Promise**: `catch` (but only total)

## Objective

Add `recover`, `recover_with`, and related combinators to the `Effect` trait that enable selective error recovery based on predicates or error patterns, maintaining zero-cost abstraction principles.

**Key Design Decision**: Use stillwater's `Predicate<Error>` trait (from `src/predicate/mod.rs`) instead of raw closures for error matching. This enables composable, reusable error predicates while maintaining zero-cost abstraction and supporting inline closures via blanket implementation.

## Requirements

### Functional Requirements

#### FR1: Effect.recover Combinator

- **MUST** provide `recover(predicate, handler)` method
- **MUST** call handler only if predicate returns true for the error
- **MUST** propagate error unchanged if predicate returns false
- **MUST** return concrete type (zero-cost)
- **MUST** use `Predicate<Error>` trait for composability

```rust
fn recover<P, H, E2>(self, predicate: P, handler: H) -> Recover<Self, P, H, E2>
where
    P: Predicate<Self::Error>,
    H: FnOnce(Self::Error) -> E2 + Send,
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;
```

**Design Rationale**:
- The predicate uses `Predicate<Error>` trait (which borrows via `.check(&error)`) since it only needs to inspect the error for matching
- The handler takes ownership of `Error` to allow extracting error details for recovery (e.g., retrying with error context)
- Closures work automatically via `Predicate`'s blanket impl for `Fn(&T) -> bool + Send + Sync`
- The `Send + Sync` bounds on predicates enable composition and reusability across threads

#### FR2: Effect.recover_with Combinator

- **MUST** provide `recover_with(predicate, handler)` where handler returns Result
- **MUST** allow returning a value directly (not wrapped in Effect)
- **MUST** support transforming to a different effect on recovery
- **MUST** use `Predicate<Error>` trait for consistency

```rust
fn recover_with<P, F>(self, predicate: P, f: F) -> RecoverWith<Self, P, F>
where
    P: Predicate<Self::Error>,
    F: FnOnce(Self::Error) -> Result<Self::Output, Self::Error> + Send;
```

#### FR3: Effect.recover_some Combinator

- **MUST** provide `recover_some(partial_fn)` using Option-returning function
- **SHOULD** feel similar to Scala's `catchSome` pattern
- **MUST** propagate error if partial_fn returns None
- **MUST** require `Error: Clone` to preserve error when partial_fn returns None

```rust
fn recover_some<F, E2>(self, f: F) -> RecoverSome<Self, F, E2>
where
    Self::Error: Clone,
    F: FnOnce(Self::Error) -> Option<E2> + Send,
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;
```

**Design Decision**: This combinator requires `Error: Clone` because the partial function consumes the error to produce `Option<E2>`, but we need to return the original error if `None` is returned. The alternatives were:
1. **Require Clone** (chosen) - Most flexible, allows handler to extract error data
2. Borrow in signature `FnOnce(&Error)` - Prevents handler from consuming error details

For non-Clone errors, users can use `recover` or `recover_with` instead.

#### FR4: Effect.fallback Combinator

- **MUST** provide `fallback(default)` for simple default value on ANY error
- **MUST** return default value directly on any error
- **MUST** accept value directly, not wrapped in Effect

```rust
fn fallback(self, default: Self::Output) -> Fallback<Self>
where
    Self::Output: Send;
```

#### FR5: Effect.fallback_to Combinator

- **MUST** provide `fallback_to(effect)` to try alternative effect on ANY error
- **SHOULD** be shorthand for `or_else(|_| alternative)`
- **MUST** accept effect directly

```rust
fn fallback_to<E2>(self, alternative: E2) -> FallbackTo<Self, E2>
where
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;
```

#### FR6: Chaining Recover Operations

- **MUST** allow chaining multiple recover calls
- Each recover handles specific error types
- Unhandled errors propagate to next recover or final error

```rust
fetch_data()
    .recover(|e| e.is_timeout(), |_| fetch_cached())
    .recover(|e| e.is_not_found(), |_| pure(default()))
    // Other errors propagate
```

### Non-Functional Requirements

#### NFR1: Zero-Cost

- Combinator types MUST NOT allocate
- Predicates and handlers SHOULD inline when possible
- Size of combinator = sum of inner sizes + discriminant

#### NFR2: Short-Circuit Semantics

- On success, recover operations MUST NOT be evaluated
- Predicate called only on error
- Handler called only if predicate returns true

#### NFR3: Type Safety

- Error types MUST match across recovery chain
- Output types MUST be consistent
- Compiler should prevent type mismatches

## Acceptance Criteria

### recover

- [ ] **AC1**: `recover` method exists on EffectExt
- [ ] **AC2**: `pure(5).recover(|_| true, |_| pure(0))` returns Ok(5) (no recovery needed)
- [ ] **AC3**: `fail("err").recover(|_| true, |_| pure(42))` returns Ok(42) (recovers)
- [ ] **AC4**: `fail("err").recover(|_| false, |_| pure(42))` returns Err("err") (predicate false)
- [ ] **AC5**: Predicate only called on error, not on success

### recover_with

- [ ] **AC6**: `recover_with` method exists
- [ ] **AC7**: Returns Result<Output, Error> for flexible recovery
- [ ] **AC8**: Can transform error or recover to value

### recover_some

- [ ] **AC9**: `recover_some` method exists
- [ ] **AC10**: `Some(effect)` from partial function recovers
- [ ] **AC11**: `None` from partial function propagates error

### fallback

- [ ] **AC12**: `fallback` method exists
- [ ] **AC13**: `fail("err").fallback(42)` returns Ok(42)
- [ ] **AC14**: `pure(5).fallback(42)` returns Ok(5)

### fallback_to

- [ ] **AC15**: `fallback_to` method exists
- [ ] **AC16**: Tries alternative effect on any error

### Chaining

- [ ] **AC17**: Multiple recover calls chain correctly
- [ ] **AC18**: First matching recover handles error
- [ ] **AC19**: Unmatched errors propagate through chain

### Integration

- [ ] **AC20**: Works with BoxedEffect via `.boxed()`
- [ ] **AC21**: Works with other combinators (map, and_then, ensure)

## Technical Details

### Implementation Approach

#### Recover Combinator Type

```rust
// src/effect/combinators/recover.rs

use crate::effect::Effect;
use crate::predicate::Predicate;
use std::marker::PhantomData;

/// Recovers from errors matching a predicate.
///
/// Zero-cost: no heap allocation. The struct stores only the inner effect,
/// predicate, and handler function.
pub struct Recover<E, P, H, E2> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) handler: H,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, P, H, E2> std::fmt::Debug for Recover<E, P, H, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Recover")
            .field("inner", &"<effect>")
            .field("predicate", &"<predicate>")
            .field("handler", &"<handler>")
            .finish()
    }
}

impl<E, P, H, E2> Recover<E, P, H, E2> {
    pub fn new(inner: E, predicate: P, handler: H) -> Self {
        Self {
            inner,
            predicate,
            handler,
            _marker: PhantomData,
        }
    }
}

impl<E, P, H, E2> Effect for Recover<E, P, H, E2>
where
    E: Effect,
    P: Predicate<E::Error>,
    H: FnOnce(E::Error) -> E2 + Send,
    E2: Effect<Output = E::Output, Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(error) => {
                if self.predicate.check(&error) {
                    (self.handler)(error).run(env).await
                } else {
                    Err(error)
                }
            }
        }
    }
}
```

#### RecoverWith Combinator Type

```rust
// src/effect/combinators/recover_with.rs

use crate::effect::Effect;
use crate::predicate::Predicate;

/// Recovers from errors with a Result-returning function.
///
/// Zero-cost: no heap allocation. Useful when recovery doesn't need
/// to run an effect, just return a value or transform the error.
pub struct RecoverWith<E, P, F> {
    pub(crate) inner: E,
    pub(crate) predicate: P,
    pub(crate) handler: F,
}

impl<E, P, F> std::fmt::Debug for RecoverWith<E, P, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoverWith")
            .field("inner", &"<effect>")
            .field("predicate", &"<predicate>")
            .field("handler", &"<handler>")
            .finish()
    }
}

impl<E, P, F> RecoverWith<E, P, F> {
    pub fn new(inner: E, predicate: P, handler: F) -> Self {
        Self {
            inner,
            predicate,
            handler,
        }
    }
}

impl<E, P, F> Effect for RecoverWith<E, P, F>
where
    E: Effect,
    P: Predicate<E::Error>,
    F: FnOnce(E::Error) -> Result<E::Output, E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(error) => {
                if self.predicate.check(&error) {
                    (self.handler)(error)
                } else {
                    Err(error)
                }
            }
        }
    }
}
```

#### RecoverSome Combinator Type

```rust
// src/effect/combinators/recover_some.rs

use crate::effect::Effect;
use std::marker::PhantomData;

/// Recovers using an Option-returning partial function.
///
/// Requires Error: Clone to preserve the error when None is returned.
/// The error is cloned before being passed to the partial function,
/// so the original can be returned if recovery is not possible.
pub struct RecoverSome<E, F, E2> {
    pub(crate) inner: E,
    pub(crate) partial_fn: F,
    pub(crate) _marker: PhantomData<E2>,
}

impl<E, F, E2> std::fmt::Debug for RecoverSome<E, F, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoverSome")
            .field("inner", &"<effect>")
            .field("partial_fn", &"<function>")
            .finish()
    }
}

impl<E, F, E2> RecoverSome<E, F, E2> {
    pub fn new(inner: E, partial_fn: F) -> Self {
        Self {
            inner,
            partial_fn,
            _marker: PhantomData,
        }
    }
}

impl<E, F, E2> Effect for RecoverSome<E, F, E2>
where
    E: Effect,
    E::Error: Clone,
    F: FnOnce(E::Error) -> Option<E2> + Send,
    E2: Effect<Output = E::Output, Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(error) => {
                // Clone error before passing to partial_fn so we can
                // return the original if None is returned
                let error_clone = error.clone();
                match (self.partial_fn)(error_clone) {
                    Some(recovery_effect) => recovery_effect.run(env).await,
                    None => Err(error), // Use original error
                }
            }
        }
    }
}
```

#### Fallback Combinator Type

```rust
// src/effect/combinators/fallback.rs

use crate::effect::Effect;

/// Provides a default value on any error.
///
/// Zero-cost: no heap allocation. Stores only the inner effect
/// and the default value.
pub struct Fallback<E> {
    pub(crate) inner: E,
    pub(crate) default: E::Output,
}

impl<E> std::fmt::Debug for Fallback<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fallback")
            .field("inner", &"<effect>")
            .field("default", &"<value>")
            .finish()
    }
}

impl<E> Fallback<E> {
    pub fn new(inner: E, default: E::Output) -> Self {
        Self { inner, default }
    }
}

impl<E> Effect for Fallback<E>
where
    E: Effect,
    E::Output: Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.inner.run(env).await {
            Ok(value) => Ok(value),
            Err(_) => Ok(self.default),
        }
    }
}
```

#### FallbackTo Combinator Type

```rust
// src/effect/combinators/fallback_to.rs

use crate::effect::Effect;

/// Tries an alternative effect on any error.
///
/// Zero-cost: no heap allocation. Stores only the primary and
/// alternative effects.
pub struct FallbackTo<E1, E2> {
    pub(crate) primary: E1,
    pub(crate) alternative: E2,
}

impl<E1, E2> std::fmt::Debug for FallbackTo<E1, E2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FallbackTo")
            .field("primary", &"<effect>")
            .field("alternative", &"<effect>")
            .finish()
    }
}

impl<E1, E2> FallbackTo<E1, E2> {
    pub fn new(primary: E1, alternative: E2) -> Self {
        Self { primary, alternative }
    }
}

impl<E1, E2> Effect for FallbackTo<E1, E2>
where
    E1: Effect,
    E2: Effect<Output = E1::Output, Error = E1::Error, Env = E1::Env>,
{
    type Output = E1::Output;
    type Error = E1::Error;
    type Env = E1::Env;

    async fn run(self, env: &Self::Env) -> Result<Self::Output, Self::Error> {
        match self.primary.run(env).await {
            Ok(value) => Ok(value),
            Err(_) => self.alternative.run(env).await,
        }
    }
}
```

#### Extension Trait Methods

```rust
// In src/effect/ext.rs

impl<E: Effect> EffectExt for E {
    /// Recover from errors matching a predicate.
    ///
    /// If the effect fails and the predicate returns true for the error,
    /// the handler is called to produce a recovery effect. If the predicate
    /// returns false, the error propagates unchanged.
    ///
    /// Uses the `Predicate<Error>` trait, which supports both closures
    /// and composable predicate combinators.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::prelude::*;
    /// use stillwater::predicate::*;
    ///
    /// #[derive(Debug, PartialEq, Clone)]
    /// enum Error {
    ///     CacheMiss,
    ///     NetworkError(String),
    /// }
    ///
    /// // Using a closure (works via blanket impl)
    /// let effect = fetch_from_cache(id)
    ///     .recover(
    ///         |e: &Error| matches!(e, Error::CacheMiss),
    ///         |_| fetch_from_db(id)
    ///     );
    ///
    /// // Or using composable predicates
    /// let is_cache_miss = |e: &Error| matches!(e, Error::CacheMiss);
    /// let is_timeout = |e: &Error| matches!(e, Error::NetworkError(s) if s.contains("timeout"));
    /// let recoverable = is_cache_miss.or(is_timeout);
    ///
    /// let effect = fetch_from_cache(id)
    ///     .recover(recoverable, |_| fetch_from_db(id));
    /// ```
    fn recover<P, H, E2>(self, predicate: P, handler: H) -> Recover<Self, P, H, E2>
    where
        P: Predicate<Self::Error>,
        H: FnOnce(Self::Error) -> E2 + Send,
        E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>,
    {
        Recover::new(self, predicate, handler)
    }

    /// Recover from errors with a Result-returning function.
    ///
    /// Similar to `recover`, but the handler returns a Result directly
    /// instead of an Effect. Useful when recovery doesn't need environment.
    ///
    /// Uses the `Predicate<Error>` trait for consistency with `recover`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::prelude::*;
    ///
    /// let effect = parse_config()
    ///     .recover_with(
    ///         |e: &ConfigError| e.is_missing_field(),
    ///         |_| Ok(Config::default())
    ///     );
    /// ```
    fn recover_with<P, F>(self, predicate: P, f: F) -> RecoverWith<Self, P, F>
    where
        P: Predicate<Self::Error>,
        F: FnOnce(Self::Error) -> Result<Self::Output, Self::Error> + Send,
    {
        RecoverWith::new(self, predicate, f)
    }

    /// Recover using a partial function.
    ///
    /// The function returns `Some(effect)` to recover, or `None` to let
    /// the error propagate. This is useful for pattern-matching on errors.
    ///
    /// Requires `Error: Clone` because the error must be cloned before
    /// being passed to the partial function, so it can be returned if
    /// `None` is produced.
    ///
    /// For non-Clone errors, use `recover` or `recover_with` instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::prelude::*;
    ///
    /// #[derive(Debug, Clone)]
    /// enum Error {
    ///     Timeout,
    ///     NotFound,
    ///     Fatal(String),
    /// }
    ///
    /// let effect = risky_operation()
    ///     .recover_some(|e| match e {
    ///         Error::Timeout => Some(pure(default_value())),
    ///         Error::NotFound => Some(create_new()),
    ///         _ => None, // Other errors propagate
    ///     });
    /// ```
    fn recover_some<F, E2>(self, f: F) -> RecoverSome<Self, F, E2>
    where
        Self::Error: Clone,
        F: FnOnce(Self::Error) -> Option<E2> + Send,
        E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>,
    {
        RecoverSome::new(self, f)
    }

    /// Provide a default value on any error.
    ///
    /// Returns the default value directly on any error without wrapping
    /// in an effect. The default value is moved into the combinator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::prelude::*;
    ///
    /// let count = get_count().fallback(0);
    /// // Returns 0 on any error
    /// ```
    fn fallback(self, default: Self::Output) -> Fallback<Self>
    where
        Self::Output: Send,
    {
        Fallback::new(self, default)
    }

    /// Try an alternative effect on any error.
    ///
    /// This is a shorthand for `or_else(|_| alternative)`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let data = fetch_primary()
    ///     .fallback_to(fetch_secondary());
    /// ```
    fn fallback_to<E2>(self, alternative: E2) -> FallbackTo<Self, E2>
    where
        E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>,
    {
        FallbackTo::new(self, alternative)
    }
}
```

### Module Structure

```
src/effect/
├── combinators/
│   ├── mod.rs
│   ├── recover.rs         # Recover<E, P, H, E2>
│   ├── recover_with.rs    # RecoverWith<E, P, F>
│   ├── recover_some.rs    # RecoverSome<E, F, E2>
│   ├── fallback.rs        # Fallback<E>
│   └── fallback_to.rs     # FallbackTo<E1, E2>
├── ext.rs                 # Add recover, recover_with, etc. methods
```

## Dependencies

### Prerequisites
- Effect trait and infrastructure (`src/effect/trait_def.rs`)
- Predicate trait (`src/predicate/mod.rs`) - for composable error matching
- Standard async runtime support

### Affected Components
- `EffectExt` trait - new methods
- `src/effect/combinators/mod.rs` - new combinator modules
- `src/effect/prelude.rs` - exports for new combinators

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::prelude::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestError {
        Recoverable(String),
        Fatal(String),
    }

    impl TestError {
        fn is_recoverable(&self) -> bool {
            matches!(self, TestError::Recoverable(_))
        }
    }

    #[tokio::test]
    async fn test_recover_on_matching_error() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover(
                |e: &TestError| e.is_recoverable(),
                |_| pure(42)
            );
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_recover_on_non_matching_error() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into()))
            .recover(
                |e| e.is_recoverable(),
                |_| pure(42)
            );
        assert_eq!(effect.execute(&()).await, Err(TestError::Fatal("boom".into())));
    }

    #[tokio::test]
    async fn test_recover_not_called_on_success() {
        let effect = pure::<_, TestError, ()>(5)
            .recover(
                |_| panic!("should not be called"),
                |_| panic!("should not be called")
            );
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    #[tokio::test]
    async fn test_recover_with_result() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover_with(
                |e| e.is_recoverable(),
                |_| Ok(42)
            );
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_recover_some_matches() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover_some(|e| match e {
                TestError::Recoverable(_) => Some(pure(42)),
                _ => None,
            });
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_recover_some_no_match() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into()))
            .recover_some(|e| match e {
                TestError::Recoverable(_) => Some(pure(42)),
                _ => None,
            });
        assert_eq!(effect.execute(&()).await, Err(TestError::Fatal("boom".into())));
    }

    #[tokio::test]
    async fn test_fallback() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into()))
            .fallback(0);
        assert_eq!(effect.execute(&()).await, Ok(0));
    }

    #[tokio::test]
    async fn test_fallback_not_used_on_success() {
        let effect = pure::<_, TestError, ()>(5)
            .fallback(0);
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    #[tokio::test]
    async fn test_fallback_to() {
        let effect = fail::<i32, _, ()>(TestError::Fatal("boom".into()))
            .fallback_to(pure(42));
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_chained_recover() {
        let effect = fail::<i32, _, ()>(TestError::Recoverable("oops".into()))
            .recover(
                |e| matches!(e, TestError::Fatal(_)),
                |_| pure(0)
            )
            .recover(
                |e| matches!(e, TestError::Recoverable(_)),
                |_| pure(42)
            );
        assert_eq!(effect.execute(&()).await, Ok(42));
    }

    #[tokio::test]
    async fn test_recover_with_other_combinators() {
        let effect = pure::<_, TestError, ()>(5)
            .map(|x| x * 2)
            .and_then(|x| {
                if x > 5 {
                    fail(TestError::Recoverable("too big".into()))
                } else {
                    pure(x)
                }
            })
            .recover(
                |e: &TestError| e.is_recoverable(),
                |_| pure(5)
            )
            .map(|x| x + 1);
        assert_eq!(effect.execute(&()).await, Ok(6));
    }

    #[tokio::test]
    async fn test_recover_with_predicate_composition() {
        use crate::predicate::PredicateExt;

        // Define reusable predicates
        let is_recoverable = |e: &TestError| matches!(e, TestError::Recoverable(_));
        let is_timeout = |e: &TestError| {
            matches!(e, TestError::Recoverable(s) if s.contains("timeout"))
        };

        // Compose predicates
        let should_retry = is_recoverable.and(is_timeout);

        let effect = fail::<i32, _, ()>(TestError::Recoverable("timeout".into()))
            .recover(should_retry, |_| pure(42));

        assert_eq!(effect.execute(&()).await, Ok(42));

        // Non-timeout recoverable errors don't match
        let effect = fail::<i32, _, ()>(TestError::Recoverable("other".into()))
            .recover(should_retry, |_| pure(42));

        assert!(effect.execute(&()).await.is_err());
    }

    #[tokio::test]
    async fn test_recover_with_or_predicate() {
        use crate::predicate::PredicateExt;

        let is_recoverable = |e: &TestError| matches!(e, TestError::Recoverable(_));
        let is_specific_fatal = |e: &TestError| {
            matches!(e, TestError::Fatal(s) if s.contains("retryable"))
        };

        // Either recoverable OR specific fatal errors
        let can_retry = is_recoverable.or(is_specific_fatal);

        // Recoverable error recovers
        let effect = fail::<i32, _, ()>(TestError::Recoverable("err".into()))
            .recover(can_retry, |_| pure(1));
        assert_eq!(effect.execute(&()).await, Ok(1));

        // Specific fatal error also recovers
        let effect = fail::<i32, _, ()>(TestError::Fatal("retryable".into()))
            .recover(can_retry, |_| pure(2));
        assert_eq!(effect.execute(&()).await, Ok(2));

        // Other fatal errors don't recover
        let effect = fail::<i32, _, ()>(TestError::Fatal("permanent".into()))
            .recover(can_retry, |_| pure(3));
        assert!(effect.execute(&()).await.is_err());
    }
}
```

### Zero-Cost Validation Tests

```rust
#[cfg(test)]
mod zero_cost_tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_recover_size_is_zero_cost() {
        use crate::effect::constructors::pure;
        use crate::effect::combinators::recover::Recover;

        type InnerEffect = Pure<i32>;
        type Predicate = fn(&String) -> bool;
        type Handler = fn(String) -> Pure<i32>;
        type RecoverEffect = Recover<InnerEffect, Predicate, Handler, Pure<i32>>;

        // Recover should be sum of inner + predicate + handler (no overhead)
        let expected = size_of::<InnerEffect>()
            + size_of::<Predicate>()
            + size_of::<Handler>()
            + size_of::<std::marker::PhantomData<Pure<i32>>>();

        assert_eq!(size_of::<RecoverEffect>(), expected);
    }

    #[test]
    fn test_fallback_size_is_zero_cost() {
        use crate::effect::constructors::pure;
        use crate::effect::combinators::fallback::Fallback;

        type InnerEffect = Pure<i32>;
        type FallbackEffect = Fallback<InnerEffect>;

        // Fallback should be sum of inner + default value
        let expected = size_of::<InnerEffect>() + size_of::<i32>();

        assert_eq!(size_of::<FallbackEffect>(), expected);
    }
}
```

## Documentation Requirements

### Code Documentation

```rust
/// Recover from errors matching a predicate.
///
/// The `recover` combinator enables selective error handling - you can
/// handle specific errors while letting others propagate. This is essential
/// for patterns like:
///
/// - **Cache fallback**: Recover from cache miss, propagate network errors
/// - **Graceful degradation**: Use defaults for non-critical failures
/// - **Retry on transient errors**: Only retry timeouts, not validation errors
///
/// # Predicate-Based Matching
///
/// Uses the `Predicate<Error>` trait from `stillwater::predicate`, which enables:
/// - **Inline closures**: `|e| e.is_timeout()`
/// - **Composable predicates**: Combine with `.and()`, `.or()`, `.not()`
/// - **Reusable matchers**: Define once, use across your codebase
///
/// # When to Use
///
/// Use `recover` when:
/// - You want to handle SOME errors but not others
/// - Recovery depends on the type or contents of the error
/// - You need to run a fallback effect on recovery
///
/// Use `recover_with` when:
/// - Recovery produces a value directly (no effect needed)
/// - You don't need access to the environment
///
/// Use `fallback` when:
/// - Any error should produce a default value
/// - You don't care about error details
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// #[derive(Debug, Clone)]
/// enum DbError {
///     NotFound,
///     ConnectionLost,
///     Timeout,
/// }
///
/// // Only recover from NotFound, let other errors propagate
/// let user = fetch_user(id)
///     .recover(
///         |e: &DbError| matches!(e, DbError::NotFound),
///         |_| create_default_user(id)
///     );
/// ```
///
/// ## Composable Predicates
///
/// ```rust
/// use stillwater::effect::prelude::*;
/// use stillwater::predicate::PredicateExt;
///
/// // Define reusable error predicates
/// let is_transient = |e: &DbError| {
///     matches!(e, DbError::Timeout | DbError::ConnectionLost)
/// };
/// let is_not_found = |e: &DbError| matches!(e, DbError::NotFound);
///
/// // Compose with .or()
/// let recoverable = is_transient.or(is_not_found);
///
/// // Use in recovery chain
/// let data = fetch_primary()
///     .recover(recoverable, |_| fetch_cached())
///     .recover(is_not_found, |_| pure(default()));
/// ```
///
/// ## Chaining Recovery Strategies
///
/// ```rust
/// let data = fetch_primary()
///     .recover(|e: &Error| e.is_timeout(), |_| fetch_cached())
///     .recover(|e: &Error| e.is_not_found(), |_| pure(default()))
///     // ConnectionLost propagates
/// ```
///
/// # See Also
///
/// - `recover_with` - for Result-returning recovery
/// - `recover_some` - for pattern-matching recovery
/// - `fallback` - for simple default values
/// - `fallback_to` - for alternative effects
/// - `or_else` - for total error handling
/// - `stillwater::predicate` - predicate combinators
```

### User Guide Addition

```markdown
## Selective Error Recovery

Stillwater provides several combinators for error recovery, powered by the
`Predicate<Error>` trait for composable, reusable error matching.

### recover - Selective Recovery

Handle specific errors while letting others propagate:

```rust
use stillwater::effect::prelude::*;

let effect = fetch_from_cache(id)
    .recover(
        |e: &Error| matches!(e, Error::CacheMiss),  // Predicate
        |_| fetch_from_db(id)                        // Handler
    );
// CacheMiss -> tries database
// NetworkError -> propagates unchanged
```

### Composable Predicates

The `Predicate` trait enables powerful error matching:

```rust
use stillwater::effect::prelude::*;
use stillwater::predicate::PredicateExt;

// Define reusable error matchers
let is_transient = |e: &ApiError| {
    matches!(e, ApiError::Timeout | ApiError::RateLimit)
};

let is_client_error = |e: &ApiError| {
    matches!(e, ApiError::BadRequest | ApiError::NotFound)
};

// Compose with .and(), .or(), .not()
let should_retry = is_transient.and(is_client_error.not());

// Use in recovery
let data = fetch_api()
    .recover(should_retry, |_| retry_with_backoff())
    .recover(is_client_error, |_| pure(default_value()));
```

### recover_some - Pattern Matching Recovery

Use Rust's pattern matching for cleaner code (requires `Error: Clone`):

```rust
let effect = risky_operation()
    .recover_some(|e| match e {
        Error::Timeout => Some(use_cached()),
        Error::NotFound => Some(create_new()),
        _ => None, // Propagate other errors
    });
```

### fallback - Simple Default

Provide a default value on any error:

```rust
let count = get_count().fallback(0);
```

### Chaining Recovery Strategies

Recovery combinators can be chained to handle different error types:

```rust
let data = fetch_primary()
    .recover(|e: &Error| e.is_timeout(), |_| fetch_cached())
    .recover(|e: &Error| e.is_rate_limited(), |_| delay_and_retry())
    .fallback(default_data());
```

### Building Error Predicate Libraries

Create reusable error predicates for your domain:

```rust
// In your error module
pub mod error_predicates {
    use crate::Error;

    pub fn is_transient(e: &Error) -> bool {
        matches!(e, Error::Timeout | Error::NetworkError(_) | Error::RateLimit)
    }

    pub fn is_permanent(e: &Error) -> bool {
        matches!(e, Error::InvalidData | Error::Unauthorized)
    }

    pub fn can_retry(e: &Error) -> bool {
        is_transient(e) && !is_permanent(e)
    }
}

// Use across your codebase
use error_predicates::*;

effect.recover(can_retry, |_| retry())
```
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Use `Predicate<Error>` trait | Enables composition, reuse, and consistency with `ensure_pred` combinator |
| Predicate borrows, handler owns | Predicate inspects via `.check(&error)`, handler extracts data - different needs |
| Handler consumes error | Allows recovery effect to use error info (e.g., retry with context) |
| `recover_some` requires Clone | Necessary to preserve error when partial function returns None |
| `fallback` does NOT require Clone | Default value is moved into combinator, used once on error path |
| Separate `fallback` combinators | Common patterns deserve ergonomic shortcuts |
| Closures work automatically | `Predicate` has blanket impl for `Fn(&T) -> bool + Send + Sync` |
| Use `async fn run` pattern | Consistent with existing combinators (Map, AndThen, OrElse) |

### Future Enhancements

1. **Error predicate library**: Built-in predicates for common error patterns (transient, permanent, retryable)
2. **`recover_async`**: Async predicate for I/O-based error classification
3. **`retry_recover`**: Combine retry with selective recovery
4. **Pattern macro**: `recover_match!` for even more ergonomic pattern matching

## Migration and Compatibility

- **Breaking changes**: None (additive)
- **New methods**: `recover`, `recover_with`, `recover_some`, `fallback`, `fallback_to`
- **New dependency**: Uses existing `Predicate<T>` trait from `src/predicate/mod.rs`
- **Backward compatibility**: Existing code unaffected; closures work via blanket impl
- **Recommended imports**: Add `use stillwater::predicate::PredicateExt;` for predicate composition

---

*"Not all errors are created equal - handle them accordingly."*
