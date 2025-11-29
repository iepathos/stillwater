---
number: 029
title: ensure/filter_or Declarative Validation Combinators
category: foundation
priority: high
status: draft
dependencies: [024, 028]
created: 2025-11-27
updated: 2025-11-28
---

# Specification 029: ensure/filter_or Declarative Validation Combinators

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect), Spec 028 (Predicate Combinators)

## Context

### The Problem

#### Primary: Effect Validation Requires Verbose Boilerplate

Effect chains currently require verbose `and_then` with if/else for every validation:

```rust
// Current: 12 lines for 2 validations (from examples/pipeline.rs)
pure::<_, String, Env>(age)
    .and_then(|a| {
        if a >= 0 {
            pure::<_, String, Env>(a).boxed()
        } else {
            fail("Age cannot be negative".to_string()).boxed()
        }
    })
    .and_then(|a| {
        if a <= 150 {
            pure::<_, String, Env>(a).boxed()
        } else {
            fail("Age cannot exceed 150".to_string()).boxed()
        }
    })
```

This pattern is:
1. **Verbose**: Each check requires 7+ lines
2. **Error-prone**: Must remember `.boxed()` and handle types correctly
3. **Hard to read**: Business logic buried in boilerplate
4. **Not composable**: Can't easily reuse validation logic

#### Secondary: Validation Type Lacks Closure Support

Stillwater 0.11.0 already provides `Validation::ensure()` but it only works with the `Predicate` trait:

```rust
// Current: Works with Predicate trait only
Validation::success("hello")
    .ensure(len_min(3), "too short")  // ✓ Works

// Does NOT work with closures
Validation::success("hello")
    .ensure(|s| s.contains('@'), "needs @")  // ✗ Error: closure not supported
```

### The Solution

Declarative combinators express intent clearly:

```rust
// Effect validation: 3 lines instead of 12
pure::<_, String, Env>(age)
    .ensure(|a| *a >= 0, "Age cannot be negative".to_string())
    .ensure(|a| *a <= 150, "Age cannot exceed 150".to_string())

// Validation with closures AND predicates
Validation::success("hello")
    .ensure(len_min(3), "too short")           // Predicate (existing)
    .ensure(|s| s.contains('@'), "needs @")    // Closure (NEW)
```

### Prior Art

- **Scala ZIO**: `filterOrFail`, `filterOrElse`
- **Rust Result**: No built-in, but common pattern
- **Java Optional**: `filter`, `orElseThrow`
- **fp-ts (TypeScript)**: `filterOrElse`

## Objective

**Primary**: Add `ensure`, `ensure_with`, and `unless` combinators to `Effect` for declarative validation in effect chains.

**Secondary**: Enhance `Validation::ensure` to accept closures in addition to the existing `Predicate` trait support.

## Comparison with Current Patterns

### Effect Type (Before/After)

**Before (Stillwater 0.11.0):**
```rust
// Verbose and_then with if/else boilerplate
fetch_user(id)
    .and_then(|user| {
        if user.age >= 18 {
            pure(user)
        } else {
            fail(Error::TooYoung)
        }
    })
    .and_then(|user| {
        if user.is_verified {
            pure(user)
        } else {
            fail(Error::NotVerified)
        }
    })
```

**After (Spec 029):**
```rust
// Declarative ensure - 2 lines
fetch_user(id)
    .ensure(|u| u.age >= 18, Error::TooYoung)
    .ensure(|u| u.is_verified, Error::NotVerified)
```

### Validation Type (Before/After)

**Before (Stillwater 0.11.0):**
```rust
// Requires Predicate trait - no closure support
use stillwater::predicate::*;

Validation::success("hello")
    .ensure(len_min(3), "too short")  // ✓ Works
    // Cannot use closures for custom validation
```

**After (Spec 029):**
```rust
// Accepts BOTH Predicates AND closures
Validation::success("hello")
    .ensure(len_min(3), "too short")           // Predicate (existing)
    .ensure(|s| s.contains('@'), "needs @")    // Closure (NEW)
```

## Requirements

### Functional Requirements

#### FR1: Effect.ensure Combinator

- **MUST** provide `ensure(predicate, error)` method on Effect
- **MUST** accept closures: `FnOnce(&Self::Output) -> bool`
- **MUST** pass value through if predicate is true
- **MUST** fail with provided error if predicate is false
- **MUST** return concrete type (zero-cost per Spec 024)

```rust
fn ensure<P, E2>(self, predicate: P, error: E2) -> Ensure<Self, P, E2>
where
    P: FnOnce(&Self::Output) -> bool + Send,
    E2: Into<Self::Error> + Send;
```

#### FR2: Effect.ensure_with Combinator

- **MUST** provide `ensure_with(predicate, error_fn)` method
- **MUST** allow error creation from the value
- **MUST** be lazy - only call error_fn if predicate fails

```rust
fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> EnsureWith<Self, P, F>
where
    P: FnOnce(&Self::Output) -> bool + Send,
    F: FnOnce(&Self::Output) -> Self::Error + Send;
```

#### FR3: Effect.unless Combinator

- **MUST** provide `unless(predicate, error)` method
- **MUST** be inverse of `ensure` (fail if predicate is TRUE)
- **SHOULD** be implemented by wrapping `Ensure` with negated predicate

```rust
fn unless<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, impl FnOnce(&Self::Output) -> bool, Err>
where
    P: FnOnce(&Self::Output) -> bool + Send,
    Err: Into<Self::Error> + Send;
```

#### FR4: Effect.filter_or Combinator

- **MUST** provide `filter_or(predicate, error)` method
- **MUST** be an alias for `ensure` (different naming convention)

#### FR5: Effect.ensure_pred Combinator

- **MUST** provide `ensure_pred(predicate, error)` method for `Predicate` trait
- **MUST** work with composed predicates from Spec 028
- **MUST** enable reusable, composable validation logic

```rust
fn ensure_pred<P, Err>(self, predicate: P, error: Err) -> EnsurePred<Self, P, Err>
where
    P: crate::predicate::Predicate<Self::Output> + Send,
    Err: Into<Self::Error> + Send;
```

#### FR6: Validation.ensure Enhancement

**Current State:** `Validation::ensure()` exists in Stillwater 0.11.0 but only accepts `Predicate` trait.

- **MUST** maintain backward compatibility with existing `Predicate` trait usage
- **SHOULD** add overload or trait bound to accept `FnOnce(&T) -> bool` closures
- **MUST** transform Success to Failure if predicate fails
- **MUST** pass Failure through unchanged

**Note:** This is an enhancement to existing functionality, not new functionality.

#### FR7: Validation.ensure_with Enhancement

- **MUST** provide `ensure_with(predicate, error_fn)` on Validation
- **MUST** work with closures for lazy error construction

#### FR8: Validation.unless Enhancement

- **MUST** provide `unless(predicate, error)` on Validation
- **MUST** be inverse of `ensure` (fail if predicate is TRUE)

#### FR9: Validation.filter_or Alias

- **SHOULD** provide `filter_or` as alias for `ensure`

### Non-Functional Requirements

#### NFR1: Zero-Cost

- Combinator types MUST NOT allocate
- Predicates SHOULD inline when possible
- Size of `Ensure<E, P, Err>` = `size_of::<E>() + size_of::<P>() + size_of::<Err>()`

#### NFR2: Ergonomics

- Method chaining SHOULD feel natural
- Type inference SHOULD work without annotations
- Closures SHOULD work directly: `.ensure(|x| x > 0, Error::Invalid)`

#### NFR3: Error Type Compatibility

- The `error` parameter MUST use `Into<Self::Error>` for flexible error types
- Documentation MUST explain error type conversion

```rust
// Can pass owned values
.ensure(|x| *x > 0, AppError::Invalid)  // AppError implements Into<Error>

// Can pass &str when error is String
.ensure(|x| *x > 0, "must be positive")  // &str Into String

// For chaining with different error types, use map_err
fetch_user(id)                                    // Error = DbError
    .map_err(AppError::from)                      // Error = AppError
    .ensure(|u| u.age >= 18, AppError::TooYoung)  // Error = AppError
```

## Acceptance Criteria

### Effect.ensure (Primary Focus)

- [ ] **AC1**: `ensure` method exists on EffectExt trait
- [ ] **AC2**: `pure(5).ensure(|x| *x > 0, "negative")` succeeds with 5
- [ ] **AC3**: `pure(-5).ensure(|x| *x > 0, "negative")` fails with "negative"
- [ ] **AC4**: `fail("error").ensure(|_| true, "other")` fails with "error" (short-circuit)
- [ ] **AC5**: `Ensure<E, P, Err>` implements Effect trait
- [ ] **AC6**: Works with closures directly (no Predicate trait required)

### Effect.ensure_with

- [ ] **AC7**: `ensure_with` method exists
- [ ] **AC8**: Error function only called when predicate fails
- [ ] **AC9**: Value accessible in error function

### Effect.unless

- [ ] **AC10**: `unless` method exists
- [ ] **AC11**: `pure(5).unless(|x| *x < 0, "positive")` succeeds
- [ ] **AC12**: `pure(-5).unless(|x| *x < 0, "negative")` fails

### Effect.ensure_pred (Predicate Integration)

- [ ] **AC13**: `ensure_pred` method exists
- [ ] **AC14**: Works with `between(18, 120)` predicate
- [ ] **AC15**: Works with composed predicates like `gt(0).and(lt(100))`

### Validation Enhancement

- [ ] **AC16**: Existing Predicate-based `ensure` still works
- [ ] **AC17**: New closure-based `ensure` works
- [ ] **AC18**: `Success(5).ensure(|x| *x > 0, "err")` returns Success(5)
- [ ] **AC19**: `Success(-5).ensure(|x| *x > 0, "err")` returns Failure("err")
- [ ] **AC20**: `Failure("e").ensure(|_| true, "other")` returns Failure("e")

### Chaining

- [ ] **AC21**: Multiple `ensure` calls chain correctly
- [ ] **AC22**: First failing ensure short-circuits
- [ ] **AC23**: Works with `map` and `and_then` in same chain

### Integration

- [ ] **AC24**: Works with Spec 028 predicates via `ensure_pred`
- [ ] **AC25**: Works with BoxedEffect via `.boxed()`

## Technical Details

### Implementation Approach

#### Ensure Combinator Type for Effect

```rust
// src/effect/combinators/ensure.rs

use crate::effect::{Effect, EffectExt};

/// Validates the effect's output with a closure predicate.
///
/// If the predicate returns true, the value passes through.
/// If the predicate returns false, the effect fails with the provided error.
pub struct Ensure<E, P, Err> {
    inner: E,
    predicate: P,
    error: Err,
}

impl<E, P, Err> Ensure<E, P, Err> {
    pub fn new(inner: E, predicate: P, error: Err) -> Self {
        Ensure { inner, predicate, error }
    }
}

impl<E, P, Err> Effect for Ensure<E, P, Err>
where
    E: Effect,
    P: FnOnce(&E::Output) -> bool + Send,
    Err: Into<E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let value = self.inner.run(env).await?;
            if (self.predicate)(&value) {
                Ok(value)
            } else {
                Err(self.error.into())
            }
        }
    }
}
```

#### EnsureWith Combinator Type

```rust
// src/effect/combinators/ensure_with.rs

/// Validates with an error factory function.
pub struct EnsureWith<E, P, F> {
    inner: E,
    predicate: P,
    error_fn: F,
}

impl<E, P, F> EnsureWith<E, P, F> {
    pub fn new(inner: E, predicate: P, error_fn: F) -> Self {
        EnsureWith { inner, predicate, error_fn }
    }
}

impl<E, P, F> Effect for EnsureWith<E, P, F>
where
    E: Effect,
    P: FnOnce(&E::Output) -> bool + Send,
    F: FnOnce(&E::Output) -> E::Error + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let value = self.inner.run(env).await?;
            if (self.predicate)(&value) {
                Ok(value)
            } else {
                Err((self.error_fn)(&value))
            }
        }
    }
}
```

#### EnsurePred Combinator for Predicate Trait

```rust
// src/effect/combinators/ensure_pred.rs

use crate::effect::Effect;
use crate::predicate::Predicate;

/// Validates using a Predicate from the predicate module.
///
/// This enables composable, reusable predicates like:
/// - `between(18, 120)`
/// - `gt(0).and(lt(100))`
/// - `len_min(3).and(len_max(20))`
pub struct EnsurePred<E, P, Err> {
    inner: E,
    predicate: P,
    error: Err,
}

impl<E, P, Err> EnsurePred<E, P, Err> {
    pub fn new(inner: E, predicate: P, error: Err) -> Self {
        EnsurePred { inner, predicate, error }
    }
}

impl<E, P, Err> Effect for EnsurePred<E, P, Err>
where
    E: Effect,
    P: Predicate<E::Output> + Send,
    Err: Into<E::Error> + Send,
{
    type Output = E::Output;
    type Error = E::Error;
    type Env = E::Env;

    fn run(self, env: &Self::Env) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send {
        async move {
            let value = self.inner.run(env).await?;
            if self.predicate.check(&value) {
                Ok(value)
            } else {
                Err(self.error.into())
            }
        }
    }
}
```

#### Extension Trait Methods

```rust
// In src/effect/ext.rs

use crate::effect::combinators::{Ensure, EnsureWith, EnsurePred};

impl<E: Effect> EffectExt for E {
    /// Ensure the output satisfies a closure predicate, failing with the given error otherwise.
    ///
    /// This is useful for adding validation to effect chains without
    /// verbose `and_then` boilerplate.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = fetch_user(id)
    ///     .ensure(|u| u.age >= 18, Error::TooYoung)
    ///     .ensure(|u| u.is_active, Error::InactiveUser);
    /// ```
    fn ensure<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        Ensure::new(self, predicate, error)
    }

    /// Ensure with a lazily-computed error.
    ///
    /// The error function is only called if the predicate fails,
    /// and receives a reference to the value.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = fetch_user(id)
    ///     .ensure_with(
    ///         |u| u.age >= 18,
    ///         |u| Error::TooYoung { actual_age: u.age }
    ///     );
    /// ```
    fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> EnsureWith<Self, P, F>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        F: FnOnce(&Self::Output) -> Self::Error + Send,
        Self: Sized,
    {
        EnsureWith::new(self, predicate, error_fn)
    }

    /// Ensure using a Predicate from the predicate module.
    ///
    /// This enables composable, reusable predicates.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::predicate::*;
    ///
    /// let valid_age = between(18, 120);
    /// let effect = fetch_age()
    ///     .ensure_pred(valid_age, Error::InvalidAge);
    /// ```
    fn ensure_pred<P, Err>(self, predicate: P, error: Err) -> EnsurePred<Self, P, Err>
    where
        P: crate::predicate::Predicate<Self::Output> + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        EnsurePred::new(self, predicate, error)
    }

    /// Alias for `ensure` - filter with a fallback error.
    ///
    /// Named to match common FP convention.
    fn filter_or<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        self.ensure(predicate, error)
    }

    /// Ensure the output does NOT satisfy a predicate.
    ///
    /// Inverse of `ensure`: fails if predicate is TRUE.
    ///
    /// Implementation: Wraps `Ensure` with negated predicate to avoid separate type.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = fetch_user(id)
    ///     .unless(|u| u.is_banned, Error::UserBanned);
    /// ```
    fn unless<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, impl FnOnce(&Self::Output) -> bool + Send, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
        Self: Sized,
    {
        Ensure::new(self, move |x| !predicate(x), error)
    }
}
```

#### Validation Enhancements

**Note:** These enhancements extend existing `Validation::ensure()` functionality from Stillwater 0.11.0.

```rust
// In src/validation/core.rs

impl<T, E> Validation<T, E> {
    /// Ensure the success value satisfies a predicate.
    ///
    /// **Enhancement**: This method exists in 0.11.0 but only accepts `Predicate` trait.
    /// This spec proposes accepting BOTH `Predicate` AND closures.
    ///
    /// Returns Failure with the provided error if the predicate fails.
    /// Passes Failure through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::predicate::*;
    ///
    /// // With Predicate (existing)
    /// let result = Validation::success("hello")
    ///     .ensure(len_min(3), "too short");
    ///
    /// // With closure (NEW)
    /// let result = Validation::success("hello")
    ///     .ensure(|s| s.contains('@'), "needs @");
    /// ```
    pub fn ensure<P>(self, predicate: P, error: E) -> Validation<T, E>
    where
        P: /* Accept both Predicate<T> AND FnOnce(&T) -> bool */,
    {
        match self {
            Validation::Success(ref value) if predicate_check(&predicate, value) => self,
            Validation::Success(_) => Validation::Failure(error),
            Validation::Failure(e) => Validation::Failure(e),
        }
    }

    /// Ensure with a lazily-computed error.
    pub fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> Validation<T, E>
    where
        P: FnOnce(&T) -> bool,
        F: FnOnce(&T) -> E,
    {
        match self {
            Validation::Success(ref value) if predicate(value) => self,
            Validation::Success(ref value) => Validation::Failure(error_fn(value)),
            Validation::Failure(e) => Validation::Failure(e),
        }
    }

    /// Alias for ensure.
    pub fn filter_or<P>(self, predicate: P, error: E) -> Validation<T, E>
    where
        P: FnOnce(&T) -> bool,
    {
        self.ensure(predicate, error)
    }

    /// Ensure the value does NOT satisfy a predicate.
    pub fn unless<P>(self, predicate: P, error: E) -> Validation<T, E>
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Validation::Success(ref value) if !predicate(value) => self,
            Validation::Success(_) => Validation::Failure(error),
            Validation::Failure(e) => Validation::Failure(e),
        }
    }
}
```

### Module Structure

```
src/effect/
├── combinators/
│   ├── mod.rs
│   ├── ensure.rs        # Ensure<E, P, Err> - closure-based
│   ├── ensure_with.rs   # EnsureWith<E, P, F> - lazy error
│   └── ensure_pred.rs   # EnsurePred<E, P, Err> - Predicate trait
├── ext.rs               # Add ensure, ensure_with, ensure_pred, unless methods
```

## Dependencies

### Prerequisites
- Spec 024 (Zero-Cost Effect Trait)
- Spec 028 (Predicate Combinators) - for `ensure_pred`

### Affected Components
- `EffectExt` trait - new methods
- `Validation` type - enhanced methods

## Testing Strategy

### Unit Tests for Effect

```rust
#[cfg(test)]
mod effect_tests {
    use super::*;

    #[tokio::test]
    async fn test_ensure_with_closure() {
        let effect = pure::<_, String, ()>(5)
            .ensure(|x| *x > 0, "must be positive".to_string());
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    #[tokio::test]
    async fn test_ensure_fails_on_false() {
        let effect = pure::<_, String, ()>(-5)
            .ensure(|x| *x > 0, "must be positive".to_string());
        assert_eq!(effect.execute(&()).await, Err("must be positive".to_string()));
    }

    #[tokio::test]
    async fn test_ensure_short_circuits_on_prior_error() {
        let effect = fail::<i32, _, ()>("prior error".to_string())
            .ensure(|_| panic!("should not be called"), "other error".to_string());
        assert_eq!(effect.execute(&()).await, Err("prior error".to_string()));
    }

    #[tokio::test]
    async fn test_ensure_with_lazy_error() {
        let effect = pure::<_, String, ()>(-5)
            .ensure_with(
                |x| *x > 0,
                |x| format!("{} is not positive", x)
            );
        assert_eq!(effect.execute(&()).await, Err("-5 is not positive".to_string()));
    }

    #[tokio::test]
    async fn test_unless_passes_on_false() {
        let effect = pure::<_, String, ()>(5)
            .unless(|x| *x < 0, "must not be negative".to_string());
        assert_eq!(effect.execute(&()).await, Ok(5));
    }

    #[tokio::test]
    async fn test_unless_fails_on_true() {
        let effect = pure::<_, String, ()>(-5)
            .unless(|x| *x < 0, "must not be negative".to_string());
        assert_eq!(effect.execute(&()).await, Err("must not be negative".to_string()));
    }

    #[tokio::test]
    async fn test_chained_ensures() {
        let effect = pure::<_, String, ()>(50)
            .ensure(|x| *x > 0, "must be positive".to_string())
            .ensure(|x| *x < 100, "must be less than 100".to_string())
            .ensure(|x| *x % 2 == 0, "must be even".to_string());
        assert_eq!(effect.execute(&()).await, Ok(50));
    }

    #[tokio::test]
    async fn test_chained_ensures_first_fails() {
        let effect = pure::<_, String, ()>(-5)
            .ensure(|x| *x > 0, "must be positive".to_string())
            .ensure(|_| panic!("should not reach"), "other".to_string());
        assert_eq!(effect.execute(&()).await, Err("must be positive".to_string()));
    }

    #[tokio::test]
    async fn test_ensure_with_map() {
        let effect = pure::<_, String, ()>(5)
            .map(|x| x * 2)
            .ensure(|x| *x > 5, "must be greater than 5".to_string())
            .map(|x| x + 1);
        assert_eq!(effect.execute(&()).await, Ok(11));
    }

    #[tokio::test]
    async fn test_ensure_with_and_then() {
        let effect = pure::<_, String, ()>(5)
            .ensure(|x| *x > 0, "must be positive".to_string())
            .and_then(|x| pure(x * 2));
        assert_eq!(effect.execute(&()).await, Ok(10));
    }
}
```

### Integration Tests with Predicates

```rust
#[cfg(test)]
mod predicate_integration {
    use super::*;
    use crate::predicate::*;

    #[tokio::test]
    async fn test_ensure_pred_with_predicate() {
        let effect = pure::<_, String, ()>(25)
            .ensure_pred(between(18, 120), "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_ensure_pred_with_composed_predicate() {
        let valid_age = gt(0).and(lt(150));

        let effect = pure::<_, String, ()>(25)
            .ensure_pred(valid_age, "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_mixing_closure_and_predicate() {
        let effect = pure::<_, String, ()>(25)
            .ensure(|x| *x > 0, "must be positive".to_string())  // Closure
            .ensure_pred(between(0, 150), "out of range".to_string());  // Predicate
        assert_eq!(effect.execute(&()).await, Ok(25));
    }
}
```

### Validation Enhancement Tests

```rust
#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::predicate::*;

    #[test]
    fn test_validation_ensure_with_predicate() {
        // Existing functionality - must still work
        let result = Validation::success(String::from("hello"))
            .ensure(len_min(3), "too short");
        assert_eq!(result, Validation::Success(String::from("hello")));
    }

    #[test]
    fn test_validation_ensure_with_closure() {
        // NEW functionality
        let result = Validation::success(String::from("hello"))
            .ensure(|s| s.contains('e'), "must contain 'e'");
        assert_eq!(result, Validation::Success(String::from("hello")));
    }

    #[test]
    fn test_validation_ensure_fails() {
        let result = Validation::<_, String>::success(-5)
            .ensure(|x| *x > 0, "must be positive".to_string());
        assert_eq!(result, Validation::Failure("must be positive".to_string()));
    }

    #[test]
    fn test_validation_ensure_preserves_failure() {
        let result = Validation::<i32, _>::failure("prior error".to_string())
            .ensure(|_| true, "other error".to_string());
        assert_eq!(result, Validation::Failure("prior error".to_string()));
    }

    #[test]
    fn test_validation_chained_ensures() {
        let result = Validation::<_, String>::success(50)
            .ensure(|x| *x > 0, "positive".to_string())
            .ensure(|x| *x < 100, "under 100".to_string());
        assert_eq!(result, Validation::Success(50));
    }

    #[test]
    fn test_validation_unless() {
        let result = Validation::<_, String>::success(5)
            .unless(|x| *x < 0, "must not be negative".to_string());
        assert_eq!(result, Validation::Success(5));

        let result = Validation::<_, String>::success(-5)
            .unless(|x| *x < 0, "must not be negative".to_string());
        assert_eq!(result, Validation::Failure("must not be negative".to_string()));
    }
}
```

## Documentation Requirements

### Code Documentation

```rust
/// Ensure the output satisfies a predicate, failing otherwise.
///
/// `ensure` is a declarative way to add validation to effect chains.
/// Instead of verbose `and_then` with pattern matching, express your
/// validation as a simple closure predicate.
///
/// # When to Use
///
/// Use `ensure` when:
/// - You want to validate a value in an effect chain
/// - The validation can be expressed as a boolean predicate
/// - You have a single error to return on failure
///
/// Use `ensure_with` when:
/// - You need to include the value in the error message
/// - The error construction is expensive and should be lazy
///
/// Use `ensure_pred` when:
/// - You want to use composable predicates from the predicate module
/// - You need reusable validation logic
///
/// Use `unless` when:
/// - You want to fail if the predicate is TRUE (inverse logic)
/// - Reads more naturally: "unless this is bad, continue"
///
/// # Example
///
/// ```rust
/// use stillwater::effect::prelude::*;
///
/// #[derive(Debug)]
/// enum Error {
///     TooYoung,
///     NotVerified,
///     Banned,
/// }
///
/// fn validate_user(user: User) -> impl Effect<Output = User, Error = Error, Env = ()> {
///     pure(user)
///         .ensure(|u| u.age >= 18, Error::TooYoung)
///         .ensure(|u| u.is_verified, Error::NotVerified)
///         .unless(|u| u.is_banned, Error::Banned)
/// }
/// ```
///
/// # See Also
///
/// - `ensure_with` - for errors that need the value
/// - `ensure_pred` - for composable predicates from predicate module
/// - `unless` - inverse of ensure (fail if TRUE)
/// - `filter_or` - alias for ensure
/// - `Validation::ensure` - same pattern for Validation type
```

### User Guide Addition

```markdown
## Declarative Validation with ensure

### Problem: Verbose Effect Validation

Effect chains currently require verbose `and_then` boilerplate:

```rust
// Verbose: 12 lines for 2 validations
pure::<_, String, Env>(age)
    .and_then(|a| {
        if a >= 0 {
            pure::<_, String, Env>(a).boxed()
        } else {
            fail("Age cannot be negative".to_string()).boxed()
        }
    })
    .and_then(|a| {
        if a <= 150 {
            pure::<_, String, Env>(a).boxed()
        } else {
            fail("Age cannot exceed 150".to_string()).boxed()
        }
    })
```

### Solution: Use `ensure`

```rust
// Clean: 3 lines
pure::<_, String, Env>(age)
    .ensure(|a| *a >= 0, "Age cannot be negative".to_string())
    .ensure(|a| *a <= 150, "Age cannot exceed 150".to_string())
```

### Chaining Multiple Validations

```rust
let effect = fetch_user(id)
    .ensure(|u| u.age >= 18, Error::TooYoung)
    .ensure(|u| u.is_verified, Error::NotVerified)
    .unless(|u| u.is_banned, Error::Banned);
```

### With Dynamic Error Messages

```rust
let effect = fetch_user(id)
    .ensure_with(
        |u| u.age >= 18,
        |u| Error::TooYoung { actual: u.age, required: 18 }
    );
```

### With Composable Predicates

```rust
use stillwater::predicate::*;

let valid_age = between(18, 120);
let has_name = is_not_empty().on(|u: &User| &u.name);

let effect = fetch_user(id)
    .ensure_pred(valid_age, Error::InvalidAge)
    .ensure_pred(has_name, Error::MissingName);
```

### Error Type Compatibility

The `error` parameter uses `Into<Self::Error>` for flexible error types:

```rust
// Can pass &str when error is String
.ensure(|x| *x > 0, "must be positive")  // &str -> String

// Can pass owned error types
.ensure(|x| *x > 0, AppError::Invalid)

// For different error types, use map_err first
fetch_user(id)                                    // Error = DbError
    .map_err(AppError::from)                      // Error = AppError
    .ensure(|u| u.age >= 18, AppError::TooYoung)  // Error = AppError
```
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Separate `ensure` and `ensure_pred` | Clear distinction: closures for inline, Predicate trait for composable |
| `ensure` vs `filter_or` | Both provided - `ensure` for Scala users, `filter_or` for Rust idiom |
| `unless` wraps `Ensure` | Simpler implementation, no separate type, same performance |
| Lazy error_fn | Avoid unnecessary allocations when predicate passes |
| Short-circuit on Failure | Consistent with `and_then` behavior |
| Effect validation primary | Biggest pain point in 0.11.0, Validation already has basic support |

### Implementation Strategy

1. **Phase 1**: Implement Effect combinators (`ensure`, `ensure_with`, `ensure_pred`, `unless`)
2. **Phase 2**: Enhance Validation to accept closures in addition to Predicates
3. **Phase 3**: Documentation and examples

### Future Enhancements

1. **`ensure_all`**: Check multiple predicates, accumulate errors
2. **Async predicates**: For validation that needs I/O
3. **Better error composition**: Chain of validation errors

## Migration and Compatibility

- **Breaking changes**: None (additive)
- **New methods on Effect**: `ensure`, `ensure_with`, `ensure_pred`, `filter_or`, `unless`
- **Enhanced methods on Validation**: Existing `ensure` accepts closures in addition to Predicates
- **Backward compatibility**: All existing Validation code continues to work

---

*"Say what you mean: if this, continue; else, fail."*
