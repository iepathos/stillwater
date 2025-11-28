---
number: 040
title: Effect recover Combinator for Partial Error Recovery
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 040: Effect recover Combinator for Partial Error Recovery

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None

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

## Requirements

### Functional Requirements

#### FR1: Effect.recover Combinator

- **MUST** provide `recover(predicate, handler)` method
- **MUST** call handler only if predicate returns true for the error
- **MUST** propagate error unchanged if predicate returns false
- **MUST** return concrete type (zero-cost)

```rust
fn recover<P, H, E2>(self, predicate: P, handler: H) -> Recover<Self, P, H>
where
    P: FnOnce(&Self::Error) -> bool + Send,
    H: FnOnce(Self::Error) -> E2 + Send,
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;
```

#### FR2: Effect.recover_with Combinator

- **MUST** provide `recover_with(predicate, handler)` where handler returns Result
- **MUST** allow returning a value directly (not wrapped in Effect)
- **MUST** support transforming to a different effect on recovery

```rust
fn recover_with<P, F, R>(self, predicate: P, f: F) -> RecoverWith<Self, P, F>
where
    P: FnOnce(&Self::Error) -> bool + Send,
    F: FnOnce(Self::Error) -> Result<Self::Output, Self::Error> + Send;
```

#### FR3: Effect.recover_some Combinator

- **MUST** provide `recover_some(partial_fn)` using Option-returning function
- **SHOULD** feel similar to Scala's `catchSome` pattern
- **MUST** propagate error if partial_fn returns None

```rust
fn recover_some<F, E2>(self, f: F) -> RecoverSome<Self, F>
where
    F: FnOnce(Self::Error) -> Option<E2> + Send,
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;
```

#### FR4: Effect.fallback Combinator

- **MUST** provide `fallback(default)` for simple default value on ANY error
- **SHOULD** be shorthand for `or_else(|_| pure(default))`
- **MUST** accept value directly, not wrapped in Effect

```rust
fn fallback(self, default: Self::Output) -> Fallback<Self>
where
    Self::Output: Clone;
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

use crate::effect::{Effect, EffectExt};
use std::future::Future;
use std::marker::PhantomData;

/// Recovers from errors matching a predicate.
pub struct Recover<E, P, H, E2> {
    inner: E,
    predicate: P,
    handler: H,
    _marker: PhantomData<E2>,
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
    P: FnOnce(&E::Error) -> bool + Send,
    H: FnOnce(E::Error) -> E2 + Send,
    E2: Effect<Output = E::Output, Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.inner.run(env).await {
                Ok(value) => Ok(value),
                Err(error) => {
                    if (self.predicate)(&error) {
                        (self.handler)(error).run(env).await
                    } else {
                        Err(error)
                    }
                }
            }
        }
    }
}
```

#### RecoverWith Combinator Type

```rust
// src/effect/combinators/recover_with.rs

/// Recovers from errors with a Result-returning function.
pub struct RecoverWith<E, P, F> {
    inner: E,
    predicate: P,
    handler: F,
}

impl<E, P, F> Effect for RecoverWith<E, P, F>
where
    E: Effect,
    P: FnOnce(&E::Error) -> bool + Send,
    F: FnOnce(E::Error) -> Result<E::Output, E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.inner.run(env).await {
                Ok(value) => Ok(value),
                Err(error) => {
                    if (self.predicate)(&error) {
                        (self.handler)(error)
                    } else {
                        Err(error)
                    }
                }
            }
        }
    }
}
```

#### RecoverSome Combinator Type

```rust
// src/effect/combinators/recover_some.rs

/// Recovers using an Option-returning partial function.
pub struct RecoverSome<E, F, E2> {
    inner: E,
    partial_fn: F,
    _marker: PhantomData<E2>,
}

impl<E, F, E2> Effect for RecoverSome<E, F, E2>
where
    E: Effect,
    F: FnOnce(E::Error) -> Option<E2> + Send,
    E2: Effect<Output = E::Output, Error = E::Error, Env = E::Env>,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.inner.run(env).await {
                Ok(value) => Ok(value),
                Err(error) => {
                    match (self.partial_fn)(error) {
                        Some(recovery_effect) => recovery_effect.run(env).await,
                        None => Err(error), // Wait, we consumed error! Need to handle this
                    }
                }
            }
        }
    }
}
```

**Note**: The `recover_some` implementation has a subtle issue - we consume the error in `partial_fn` but need it back if `None` is returned. Two solutions:

1. **Clone the error** before calling partial_fn (requires `E::Error: Clone`)
2. **Change signature** to take `&Error` and return the error back on None

Preferred solution (option 2):

```rust
fn recover_some<F, E2>(self, f: F) -> RecoverSome<Self, F>
where
    F: FnOnce(&Self::Error) -> Option<E2> + Send,
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;
```

But then handler can't consume error. Alternative:

```rust
fn recover_some<F, E2>(self, f: F) -> RecoverSome<Self, F>
where
    Self::Error: Clone,
    F: FnOnce(Self::Error) -> Option<E2> + Send,
    E2: Effect<Output = Self::Output, Error = Self::Error, Env = Self::Env>;

// Implementation clones error, passes to f, uses original on None
```

#### Fallback Combinator Type

```rust
// src/effect/combinators/fallback.rs

/// Provides a default value on any error.
pub struct Fallback<E> {
    inner: E,
    default: E::Output,
}

impl<E> Effect for Fallback<E>
where
    E: Effect,
    E::Output: Clone + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.inner.run(env).await {
                Ok(value) => Ok(value),
                Err(_) => Ok(self.default),
            }
        }
    }
}
```

#### FallbackTo Combinator Type

```rust
// src/effect/combinators/fallback_to.rs

/// Tries an alternative effect on any error.
pub struct FallbackTo<E1, E2> {
    primary: E1,
    alternative: E2,
}

impl<E1, E2> Effect for FallbackTo<E1, E2>
where
    E1: Effect,
    E2: Effect<Output = E1::Output, Error = E1::Error, Env = E1::Env>,
{
    type Output = E1::Output;
    type Error = E1::Error;
    type Env = E1::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            match self.primary.run(env).await {
                Ok(value) => Ok(value),
                Err(_) => self.alternative.run(env).await,
            }
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
    /// # Example
    ///
    /// ```rust
    /// use stillwater::effect::prelude::*;
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum Error {
    ///     CacheMiss,
    ///     NetworkError(String),
    /// }
    ///
    /// let effect = fetch_from_cache(id)
    ///     .recover(
    ///         |e| matches!(e, Error::CacheMiss),
    ///         |_| fetch_from_db(id)
    ///     );
    /// // CacheMiss: tries database
    /// // NetworkError: propagates
    /// ```
    fn recover<P, H, E2>(self, predicate: P, handler: H) -> Recover<Self, P, H, E2>
    where
        P: FnOnce(&Self::Error) -> bool + Send,
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
    /// # Example
    ///
    /// ```rust
    /// let effect = parse_config()
    ///     .recover_with(
    ///         |e| e.is_missing_field(),
    ///         |e| Ok(Config::default())
    ///     );
    /// ```
    fn recover_with<P, F>(self, predicate: P, f: F) -> RecoverWith<Self, P, F>
    where
        P: FnOnce(&Self::Error) -> bool + Send,
        F: FnOnce(Self::Error) -> Result<Self::Output, Self::Error> + Send,
    {
        RecoverWith::new(self, predicate, f)
    }

    /// Recover using a partial function.
    ///
    /// The function returns `Some(effect)` to recover, or `None` to let
    /// the error propagate. This is useful for pattern-matching on errors.
    ///
    /// # Example
    ///
    /// ```rust
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
    /// This is a shorthand for `or_else(|_| pure(default))`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let count = get_count().fallback(0);
    /// // Returns 0 on any error
    /// ```
    fn fallback(self, default: Self::Output) -> Fallback<Self>
    where
        Self::Output: Clone + Send,
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
- None (builds on existing Effect infrastructure)

### Affected Components
- `EffectExt` trait - new methods

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
                |e| e.is_recoverable(),
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
                |e| e.is_recoverable(),
                |_| pure(5)
            )
            .map(|x| x + 1);
        assert_eq!(effect.execute(&()).await, Ok(6));
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
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// enum DbError {
///     NotFound,
///     ConnectionLost,
///     Timeout,
/// }
///
/// // Only recover from NotFound, let other errors propagate
/// let user = fetch_user(id)
///     .recover(
///         |e| matches!(e, DbError::NotFound),
///         |_| create_default_user(id)
///     );
///
/// // Chain multiple recovery strategies
/// let data = fetch_primary()
///     .recover(|e| e.is_timeout(), |_| fetch_cached())
///     .recover(|e| e.is_not_found(), |_| pure(default()))
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
```

### User Guide Addition

```markdown
## Selective Error Recovery

Stillwater provides several combinators for error recovery:

### recover - Selective Recovery

Handle specific errors while letting others propagate:

```rust
let effect = fetch_from_cache(id)
    .recover(
        |e| matches!(e, Error::CacheMiss),  // Predicate
        |_| fetch_from_db(id)                // Handler
    );
// CacheMiss -> tries database
// NetworkError -> propagates unchanged
```

### recover_some - Pattern Matching Recovery

Use Rust's pattern matching for cleaner code:

```rust
let effect = risky_operation()
    .recover_some(|e| match e {
        Error::Timeout => Some(use_cached()),
        Error::NotFound => Some(create_new()),
        _ => None, // Propagate
    });
```

### fallback - Simple Default

Provide a default value on any error:

```rust
let count = get_count().fallback(0);
```

### Chaining Recovery Strategies

Recovery combinators can be chained:

```rust
let data = fetch_primary()
    .recover(|e| e.is_timeout(), |_| fetch_cached())
    .recover(|e| e.is_rate_limited(), |_| delay_and_retry())
    .fallback(default_data());
```
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Predicate-based API | More flexible than type-based matching |
| Handler consumes error | Allows recovery effect to use error info |
| `recover_some` requires Clone | Necessary to preserve error on None |
| Separate `fallback` combinators | Common patterns deserve shortcuts |

### Future Enhancements

1. **`recover_async`**: Async predicate for I/O-based error classification
2. **`retry_recover`**: Combine retry with selective recovery
3. **Pattern macro**: `recover_match!` for ergonomic pattern matching

## Migration and Compatibility

- **Breaking changes**: None (additive)
- **New methods**: `recover`, `recover_with`, `recover_some`, `fallback`, `fallback_to`

---

*"Not all errors are created equal - handle them accordingly."*
