---
number: 029
title: ensure/filter_or Declarative Validation Combinators
category: foundation
priority: high
status: draft
dependencies: [024, 028]
created: 2025-11-27
---

# Specification 029: ensure/filter_or Declarative Validation Combinators

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: Spec 024 (Zero-Cost Effect), Spec 028 (Predicate Combinators)

## Context

### The Problem

Validation in Effect chains currently requires verbose pattern matching or awkward `and_then` calls:

```rust
// Current: Verbose and error-prone
let effect = fetch_user(id)
    .and_then(|user| {
        if user.age >= 18 {
            Effect::pure(user)
        } else {
            Effect::fail(Error::TooYoung)
        }
    })
    .and_then(|user| {
        if user.is_verified {
            Effect::pure(user)
        } else {
            Effect::fail(Error::NotVerified)
        }
    });
```

This pattern is:
1. **Verbose**: Each check requires 5+ lines
2. **Error-prone**: Easy to swap Ok/Err branches
3. **Hard to read**: Business logic buried in boilerplate
4. **Not composable**: Can't easily reuse validation logic

### The Solution

Declarative combinators express intent clearly:

```rust
// With ensure/filter_or: Clear, concise, declarative
let effect = fetch_user(id)
    .ensure(|u| u.age >= 18, Error::TooYoung)
    .ensure(|u| u.is_verified, Error::NotVerified);

// Or with predicates from Spec 028
let effect = fetch_user(id)
    .filter_or(predicate::ge(18).on(|u| u.age), Error::TooYoung)
    .filter_or(|u| u.is_verified, Error::NotVerified);
```

### Prior Art

- **Scala ZIO**: `filterOrFail`, `filterOrElse`
- **Rust Result**: No built-in, but common pattern
- **Java Optional**: `filter`, `orElseThrow`
- **fp-ts (TypeScript)**: `filterOrElse`

## Objective

Add `ensure` and `filter_or` combinators to `Effect` and enhance `Validation` with similar declarative validation methods that integrate with the predicate combinator system from Spec 028.

## Requirements

### Functional Requirements

#### FR1: Effect.ensure Combinator

- **MUST** provide `ensure(predicate, error)` method on Effect
- **MUST** pass value through if predicate is true
- **MUST** fail with provided error if predicate is false
- **MUST** work with closures and `Predicate` trait
- **MUST** return concrete type (zero-cost per Spec 024)

```rust
fn ensure<P, E2>(self, predicate: P, error: E2) -> Ensure<Self, P, E2>
where
    P: FnOnce(&Self::Output) -> bool + Send,
    E2: Into<Self::Error>;
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

#### FR3: Effect.filter_or Combinator

- **MUST** provide `filter_or(predicate, error)` method
- **MUST** be an alias/equivalent to `ensure` (different naming convention)
- **SHOULD** integrate with Spec 028 predicates

#### FR4: Effect.unless Combinator

- **MUST** provide `unless(predicate, error)` method
- **MUST** be inverse of `ensure` (fail if predicate is TRUE)
- **MUST** be equivalent to `ensure(|x| !predicate(x), error)`

#### FR5: Validation.ensure Combinator

- **MUST** provide `ensure(predicate, error)` on Validation
- **MUST** transform Success to Failure if predicate fails
- **MUST** pass Failure through unchanged
- **SHOULD** integrate with Spec 028 predicates

```rust
impl<T, E> Validation<T, E> {
    fn ensure<P>(self, predicate: P, error: E) -> Validation<T, E>
    where
        P: FnOnce(&T) -> bool;
}
```

#### FR6: Validation.filter Combinator

- **MUST** provide `filter(predicate)` returning `Option<Validation<T, E>>`
- **MUST** return `None` if predicate fails on Success
- **SHOULD** provide `filter_or` as alias for `ensure`

#### FR7: Validation Chaining

- **MUST** allow chaining multiple ensures
- **MUST** short-circuit on first failure (fail-fast)
- **SHOULD** provide `ensure_all` for accumulating errors

```rust
// Fail-fast chaining
Validation::success(value)
    .ensure(p1, e1)
    .ensure(p2, e2)  // Not checked if p1 fails

// Error accumulation variant
Validation::success(value)
    .ensure_all([
        (p1, e1),
        (p2, e2),
    ])  // Checks all, accumulates errors
```

### Non-Functional Requirements

#### NFR1: Zero-Cost

- Combinator types MUST NOT allocate
- Predicates SHOULD inline when possible
- Size of `Ensure<E, P, Err>` = `size_of::<E>() + size_of::<P>() + size_of::<Err>()`

#### NFR2: Ergonomics

- Method chaining SHOULD feel natural
- Type inference SHOULD work without annotations
- Closures SHOULD work directly: `.ensure(|x| x > 0, Error::Invalid)`

#### NFR3: Error Messages

- Compiler errors SHOULD be clear when types don't match
- Runtime panic messages SHOULD be helpful

## Acceptance Criteria

### Effect.ensure

- [ ] **AC1**: `ensure` method exists on EffectExt trait
- [ ] **AC2**: `pure(5).ensure(|x| *x > 0, "negative")` succeeds with 5
- [ ] **AC3**: `pure(-5).ensure(|x| *x > 0, "negative")` fails with "negative"
- [ ] **AC4**: `fail("error").ensure(|_| true, "other")` fails with "error" (short-circuit)
- [ ] **AC5**: `Ensure<E, P, Err>` implements Effect trait

### Effect.ensure_with

- [ ] **AC6**: `ensure_with` method exists
- [ ] **AC7**: Error function only called when predicate fails
- [ ] **AC8**: Value accessible in error function

### Effect.unless

- [ ] **AC9**: `unless` method exists
- [ ] **AC10**: `pure(5).unless(|x| *x < 0, "positive")` succeeds
- [ ] **AC11**: `pure(-5).unless(|x| *x < 0, "negative")` fails

### Validation.ensure

- [ ] **AC12**: `ensure` method exists on Validation
- [ ] **AC13**: `Success(5).ensure(|x| *x > 0, "err")` returns Success(5)
- [ ] **AC14**: `Success(-5).ensure(|x| *x > 0, "err")` returns Failure("err")
- [ ] **AC15**: `Failure("e").ensure(|_| true, "other")` returns Failure("e")

### Chaining

- [ ] **AC16**: Multiple `ensure` calls chain correctly
- [ ] **AC17**: First failing ensure short-circuits
- [ ] **AC18**: Works with `map` and `and_then` in same chain

### Integration

- [ ] **AC19**: Works with Spec 028 predicates
- [ ] **AC20**: Works with BoxedEffect via `.boxed()`

## Technical Details

### Implementation Approach

#### Ensure Combinator Type for Effect

```rust
// src/effect/combinators/ensure.rs

use crate::effect::{Effect, EffectExt};
use std::marker::PhantomData;

/// Validates the effect's output with a predicate.
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

#### Extension Trait Methods

```rust
// In src/effect/ext.rs

impl<E: Effect> EffectExt for E {
    /// Ensure the output satisfies a predicate, failing with the given error otherwise.
    ///
    /// This is useful for adding validation to effect chains without
    /// verbose pattern matching.
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
    {
        EnsureWith::new(self, predicate, error_fn)
    }

    /// Alias for `ensure` - filter with a fallback error.
    ///
    /// Named to match common FP convention.
    fn filter_or<P, Err>(self, predicate: P, error: Err) -> Ensure<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
    {
        self.ensure(predicate, error)
    }

    /// Ensure the output does NOT satisfy a predicate.
    ///
    /// Inverse of `ensure`: fails if predicate is TRUE.
    ///
    /// # Example
    ///
    /// ```rust
    /// let effect = fetch_user(id)
    ///     .unless(|u| u.is_banned, Error::UserBanned);
    /// ```
    fn unless<P, Err>(self, predicate: P, error: Err) -> Unless<Self, P, Err>
    where
        P: FnOnce(&Self::Output) -> bool + Send,
        Err: Into<Self::Error> + Send,
    {
        Unless::new(self, predicate, error)
    }
}
```

#### Unless Combinator Type

```rust
// src/effect/combinators/unless.rs

/// Fails if the predicate returns TRUE.
pub struct Unless<E, P, Err> {
    inner: E,
    predicate: P,
    error: Err,
}

impl<E, P, Err> Effect for Unless<E, P, Err>
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
            if !(self.predicate)(&value) {
                Ok(value)
            } else {
                Err(self.error.into())
            }
        }
    }
}
```

#### Validation Methods

```rust
// In src/validation/core.rs

impl<T, E> Validation<T, E> {
    /// Ensure the success value satisfies a predicate.
    ///
    /// Returns Failure with the provided error if the predicate fails.
    /// Passes Failure through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// let result = Validation::success(5)
    ///     .ensure(|x| *x > 0, "must be positive")
    ///     .ensure(|x| *x < 100, "must be less than 100");
    /// ```
    pub fn ensure<P>(self, predicate: P, error: E) -> Validation<T, E>
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Validation::Success(ref value) if predicate(value) => self,
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

    /// Filter, returning None if predicate fails.
    ///
    /// Unlike `ensure`, this doesn't require an error - it just
    /// converts the Validation to Option<Validation>.
    pub fn filter<P>(self, predicate: P) -> Option<Validation<T, E>>
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Validation::Success(ref value) if predicate(value) => Some(self),
            Validation::Success(_) => None,
            Validation::Failure(_) => Some(self),
        }
    }
}
```

#### Integration with Predicate Combinators (Spec 028)

```rust
// In src/effect/ext.rs - additional methods

impl<E: Effect> EffectExt for E {
    /// Ensure using a Predicate from the predicate module.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::predicate::*;
    ///
    /// let effect = fetch_age()
    ///     .ensure_pred(between(18, 120), Error::InvalidAge);
    /// ```
    fn ensure_pred<P, Err>(self, predicate: P, error: Err) -> EnsurePred<Self, P, Err>
    where
        P: crate::predicate::Predicate<Self::Output> + Send,
        Err: Into<Self::Error> + Send,
    {
        EnsurePred::new(self, predicate, error)
    }
}

// Combinator that uses Predicate trait
pub struct EnsurePred<E, P, Err> {
    inner: E,
    predicate: P,
    error: Err,
}

impl<E, P, Err> Effect for EnsurePred<E, P, Err>
where
    E: Effect,
    P: crate::predicate::Predicate<E::Output> + Send,
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

### Module Structure

```
src/effect/
├── combinators/
│   ├── mod.rs
│   ├── ensure.rs        # Ensure<E, P, Err>
│   ├── ensure_with.rs   # EnsureWith<E, P, F>
│   ├── ensure_pred.rs   # EnsurePred<E, P, Err>
│   └── unless.rs        # Unless<E, P, Err>
├── ext.rs               # Add ensure, ensure_with, unless methods
```

## Dependencies

### Prerequisites
- Spec 024 (Zero-Cost Effect Trait)
- Spec 028 (Predicate Combinators) - for `ensure_pred`

### Affected Components
- `EffectExt` trait - new methods
- `Validation` type - new methods

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod effect_tests {
    use super::*;

    #[tokio::test]
    async fn test_ensure_passes_on_true() {
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

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_validation_ensure_passes() {
        let result = Validation::<_, String>::success(5)
            .ensure(|x| *x > 0, "must be positive".to_string());
        assert_eq!(result, Validation::Success(5));
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
    fn test_validation_filter() {
        let result = Validation::<_, String>::success(5)
            .filter(|x| *x > 0);
        assert!(result.is_some());

        let result = Validation::<_, String>::success(-5)
            .filter(|x| *x > 0);
        assert!(result.is_none());
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

### Integration Tests with Predicates

```rust
#[cfg(test)]
mod predicate_integration {
    use super::*;
    use crate::predicate::*;

    #[tokio::test]
    async fn test_ensure_with_predicate() {
        let effect = pure::<_, String, ()>(25)
            .ensure_pred(between(18, 120), "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Ok(25));
    }

    #[tokio::test]
    async fn test_ensure_with_combined_predicate() {
        let valid_age = between(0, 150).and(|age: &i32| *age != 69);

        let effect = pure::<_, String, ()>(25)
            .ensure_pred(valid_age, "invalid age".to_string());
        assert_eq!(effect.execute(&()).await, Ok(25));
    }

    #[test]
    fn test_validation_with_predicate() {
        let result = Validation::<_, String>::success("hello")
            .ensure(|s| len_between(3, 10).check(*s), "invalid length".to_string());
        assert_eq!(result, Validation::Success("hello"));
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
/// validation as a simple predicate.
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
/// - `unless` - inverse of ensure (fail if TRUE)
/// - `filter_or` - alias for ensure
/// - `Validation::ensure` - same pattern for Validation type
```

### User Guide Addition

```markdown
## Declarative Validation with ensure

Instead of verbose pattern matching in effect chains:

```rust
// Verbose
let effect = fetch_user(id)
    .and_then(|user| {
        if user.age >= 18 {
            pure(user)
        } else {
            fail(Error::TooYoung)
        }
    });
```

Use `ensure` for clean, declarative validation:

```rust
// Clean
let effect = fetch_user(id)
    .ensure(|u| u.age >= 18, Error::TooYoung);
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
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| `ensure` vs `filter_or` | Both provided - `ensure` for Scala users, `filter_or` for Rust idiom |
| `unless` as separate | More readable than `ensure(|x| !pred(x), ...)` |
| Lazy error_fn | Avoid unnecessary allocations when predicate passes |
| Short-circuit on Failure | Consistent with `and_then` behavior |

### Future Enhancements

1. **`ensure_all`**: Check multiple predicates, accumulate errors
2. **Async predicates**: For validation that needs I/O
3. **Better error composition**: Chain of validation errors

## Migration and Compatibility

- **Breaking changes**: None (additive)
- **New methods**: `ensure`, `ensure_with`, `filter_or`, `unless` on Effect and Validation

---

*"Say what you mean: if this, continue; else, fail."*
