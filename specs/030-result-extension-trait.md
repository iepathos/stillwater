---
number: 030
title: Result Extension Trait
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 030: Result Extension Trait

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

### The Problem

Stillwater provides rich combinators for `Effect` and `Validation`, but standard Rust `Result` lacks some useful functional programming operations. Since Stillwater integrates deeply with `Result` (Effect runs return `Result`, conversions between types), users often work with both.

Currently, users must either:
1. Convert `Result` to `Effect`/`Validation` to use combinators
2. Write verbose match expressions for simple operations
3. Import external crates like `tap` for basic functionality

```rust
// Current: Awkward for simple operations
let result: Result<User, Error> = fetch_user();

// Want to log but keep the result - no built-in way
let result = match &result {
    Ok(user) => { log::info!("Got user: {}", user.name); }
    Err(e) => { log::error!("Failed: {}", e); }
};

// Want to ensure a condition - requires conversion or verbose match
let validated = result.and_then(|user| {
    if user.age >= 18 {
        Ok(user)
    } else {
        Err(Error::TooYoung)
    }
});
```

### The Solution

A `ResultExt` trait adds useful combinators to `Result`:

```rust
use stillwater::ResultExt;

let result = fetch_user()
    .tap_ok(|user| log::info!("Got user: {}", user.name))
    .tap_err(|e| log::error!("Failed: {}", e))
    .ensure(|user| user.age >= 18, Error::TooYoung)
    .context("fetching user profile");
```

### Prior Art

- **tap crate**: `tap_ok`, `tap_err` for side effects
- **anyhow/eyre**: `context`, `with_context`
- **Scala**: Rich Result-like operations on `Either`/`Try`
- **Haskell**: Many combinators in `Control.Monad`

## Objective

Add a `ResultExt` extension trait that enhances `std::result::Result` with functional programming combinators, providing seamless integration with Stillwater's ecosystem while remaining useful as standalone utilities.

## Requirements

### Functional Requirements

#### FR1: Side Effect Combinators (tap)

- **MUST** provide `tap_ok<F>(self, f: F) -> Self` for side effects on Ok
- **MUST** provide `tap_err<F>(self, f: F) -> Self` for side effects on Err
- **MUST** provide `tap<F, G>(self, ok_fn: F, err_fn: G) -> Self` for both
- **MUST** NOT consume the value (pass through unchanged)
- **MUST** NOT change the result type

```rust
fn tap_ok<F>(self, f: F) -> Self
where
    F: FnOnce(&T);

fn tap_err<F>(self, f: F) -> Self
where
    F: FnOnce(&E);
```

#### FR2: Validation Combinators

- **MUST** provide `ensure<P, E2>(self, predicate: P, error: E2) -> Result<T, E>`
- **MUST** provide `ensure_with<P, F>(self, predicate: P, error_fn: F) -> Result<T, E>`
- **MUST** provide `unless<P, E2>(self, predicate: P, error: E2) -> Result<T, E>`
- **MUST** pass Err through unchanged
- **SHOULD** integrate with Spec 028 predicates

```rust
fn ensure<P, E2>(self, predicate: P, error: E2) -> Result<T, E>
where
    P: FnOnce(&T) -> bool,
    E2: Into<E>;
```

#### FR3: Context/Error Wrapping

- **MUST** provide `context<C>(self, context: C) -> Result<T, ContextError<E>>`
- **MUST** provide `with_context<C, F>(self, f: F) -> Result<T, ContextError<E>>`
- **SHOULD** integrate with Stillwater's `ContextError` type
- **SHOULD** provide `map_context` for existing ContextError results

```rust
fn context<C: Display>(self, context: C) -> Result<T, ContextError<E>>;
```

#### FR4: Conversion Combinators

- **MUST** provide `to_validation(self) -> Validation<T, E>`
- **MUST** provide `to_either(self) -> Either<E, T>`
- **MUST** provide `to_effect(self) -> Effect<T, E, Env>` (for any Env)
- **SHOULD** provide `swap(self) -> Result<E, T>`

#### FR5: Option-like Combinators

- **MUST** provide `ok_or_else_with<F>(self, f: F) -> T` (like `unwrap_or_else`)
- **MUST** provide `contains<U>(self, value: &U) -> bool` where T: PartialEq<U>
- **MUST** provide `contains_err<U>(self, value: &U) -> bool` where E: PartialEq<U>

#### FR6: Flattening

- **MUST** provide `flatten(self) -> Result<T, E>` for `Result<Result<T, E>, E>`
- **MUST** provide `transpose(self)` for `Result<Option<T>, E>` -> `Option<Result<T, E>>`

### Non-Functional Requirements

#### NFR1: Zero-Cost

- All combinators MUST be `#[inline]`
- No heap allocation for basic operations
- Equivalent to hand-written match statements

#### NFR2: Compatibility

- MUST NOT conflict with std Result methods
- MUST work with any Result (not tied to Stillwater error types)
- SHOULD be usable independently of other Stillwater features

#### NFR3: Ergonomics

- Import with single `use stillwater::ResultExt`
- Method names should be intuitive
- Type inference should work naturally

## Acceptance Criteria

### tap Combinators

- [ ] **AC1**: `Ok(5).tap_ok(|x| println!("{}", x))` prints "5" and returns `Ok(5)`
- [ ] **AC2**: `Err("e").tap_ok(|_| panic!())` returns `Err("e")` without panicking
- [ ] **AC3**: `Err("e").tap_err(|e| println!("{}", e))` prints "e" and returns `Err("e")`
- [ ] **AC4**: `Ok(5).tap_err(|_| panic!())` returns `Ok(5)` without panicking

### ensure Combinators

- [ ] **AC5**: `Ok(5).ensure(|x| *x > 0, "negative")` returns `Ok(5)`
- [ ] **AC6**: `Ok(-5).ensure(|x| *x > 0, "negative")` returns `Err("negative")`
- [ ] **AC7**: `Err("prior").ensure(|_| false, "other")` returns `Err("prior")`
- [ ] **AC8**: `Ok(5).unless(|x| *x < 0, "negative")` returns `Ok(5)`

### context Combinators

- [ ] **AC9**: `Err("io error").context("reading config")` wraps error with context
- [ ] **AC10**: `Ok(5).context("reading config")` returns `Ok(5)` unchanged
- [ ] **AC11**: Context chain is preserved through multiple calls

### Conversion Combinators

- [ ] **AC12**: `Ok(5).to_validation()` returns `Validation::Success(5)`
- [ ] **AC13**: `Err("e").to_validation()` returns `Validation::Failure("e")`
- [ ] **AC14**: `Ok(5).to_either()` returns `Either::Right(5)`
- [ ] **AC15**: `Err("e").to_either()` returns `Either::Left("e")`

### Other Combinators

- [ ] **AC16**: `Ok(5).contains(&5)` returns `true`
- [ ] **AC17**: `Ok(5).contains(&6)` returns `false`
- [ ] **AC18**: `Ok(Ok(5)).flatten()` returns `Ok(5)`
- [ ] **AC19**: `Ok(Err("e")).flatten()` returns `Err("e")`

## Technical Details

### Implementation Approach

#### ResultExt Trait Definition

```rust
// src/result_ext.rs

use crate::{ContextError, Either, Validation};
use std::fmt::Display;

/// Extension trait for `Result` with functional programming combinators.
///
/// This trait adds useful operations to `Result` that integrate well with
/// Stillwater's ecosystem while being useful on their own.
///
/// # Example
///
/// ```rust
/// use stillwater::ResultExt;
///
/// let result: Result<i32, &str> = Ok(42);
/// let processed = result
///     .tap_ok(|x| println!("Got: {}", x))
///     .ensure(|x| *x > 0, "must be positive")
///     .context("processing value");
/// ```
pub trait ResultExt<T, E> {
    // ========== Side Effects (tap) ==========

    /// Perform a side effect on the Ok value without consuming it.
    ///
    /// The function receives a reference to the value, and the original
    /// Result is returned unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::ResultExt;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// result.tap_ok(|x| println!("Got value: {}", x));
    /// // Prints: Got value: 42
    /// ```
    fn tap_ok<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);

    /// Perform a side effect on the Err value without consuming it.
    fn tap_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E);

    /// Perform side effects on both variants.
    fn tap<F, G>(self, ok_fn: F, err_fn: G) -> Self
    where
        F: FnOnce(&T),
        G: FnOnce(&E);

    // ========== Validation ==========

    /// Ensure the Ok value satisfies a predicate.
    ///
    /// Returns `Err(error)` if the predicate returns false.
    /// Passes `Err` through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::ResultExt;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// let checked = result.ensure(|x| *x > 0, "must be positive");
    /// assert_eq!(checked, Ok(42));
    ///
    /// let result: Result<i32, &str> = Ok(-5);
    /// let checked = result.ensure(|x| *x > 0, "must be positive");
    /// assert_eq!(checked, Err("must be positive"));
    /// ```
    fn ensure<P, E2>(self, predicate: P, error: E2) -> Result<T, E>
    where
        P: FnOnce(&T) -> bool,
        E2: Into<E>;

    /// Ensure with a lazily-computed error.
    fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> Result<T, E>
    where
        P: FnOnce(&T) -> bool,
        F: FnOnce(&T) -> E;

    /// Ensure the Ok value does NOT satisfy a predicate.
    ///
    /// Returns `Err(error)` if the predicate returns TRUE.
    fn unless<P, E2>(self, predicate: P, error: E2) -> Result<T, E>
    where
        P: FnOnce(&T) -> bool,
        E2: Into<E>;

    // ========== Context ==========

    /// Add context to an error.
    ///
    /// Wraps the error in a `ContextError` with the provided context message.
    /// Ok values pass through unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::ResultExt;
    ///
    /// fn read_config() -> Result<Config, std::io::Error> {
    ///     // ...
    /// }
    ///
    /// let config = read_config().context("loading configuration file");
    /// ```
    fn context<C>(self, context: C) -> Result<T, ContextError<E>>
    where
        C: Display + Send + Sync + 'static;

    /// Add context with a lazily-evaluated message.
    fn with_context<C, F>(self, f: F) -> Result<T, ContextError<E>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    // ========== Conversions ==========

    /// Convert to Validation (Ok -> Success, Err -> Failure).
    fn to_validation(self) -> Validation<T, E>;

    /// Convert to Either (Ok -> Right, Err -> Left).
    fn to_either(self) -> Either<E, T>;

    /// Swap Ok and Err.
    fn swap(self) -> Result<E, T>;

    // ========== Queries ==========

    /// Check if Ok contains a value equal to the given value.
    fn contains<U>(&self, value: &U) -> bool
    where
        T: PartialEq<U>;

    /// Check if Err contains a value equal to the given value.
    fn contains_err<U>(&self, value: &U) -> bool
    where
        E: PartialEq<U>;
}

// Implementation
impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[inline]
    fn tap_ok<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref value) = self {
            f(value);
        }
        self
    }

    #[inline]
    fn tap_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref error) = self {
            f(error);
        }
        self
    }

    #[inline]
    fn tap<F, G>(self, ok_fn: F, err_fn: G) -> Self
    where
        F: FnOnce(&T),
        G: FnOnce(&E),
    {
        match &self {
            Ok(value) => ok_fn(value),
            Err(error) => err_fn(error),
        }
        self
    }

    #[inline]
    fn ensure<P, E2>(self, predicate: P, error: E2) -> Result<T, E>
    where
        P: FnOnce(&T) -> bool,
        E2: Into<E>,
    {
        match self {
            Ok(ref value) if predicate(value) => self,
            Ok(_) => Err(error.into()),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> Result<T, E>
    where
        P: FnOnce(&T) -> bool,
        F: FnOnce(&T) -> E,
    {
        match self {
            Ok(ref value) if predicate(value) => self,
            Ok(ref value) => Err(error_fn(value)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn unless<P, E2>(self, predicate: P, error: E2) -> Result<T, E>
    where
        P: FnOnce(&T) -> bool,
        E2: Into<E>,
    {
        match self {
            Ok(ref value) if !predicate(value) => self,
            Ok(_) => Err(error.into()),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn context<C>(self, context: C) -> Result<T, ContextError<E>>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| ContextError::new(e, context.to_string()))
    }

    #[inline]
    fn with_context<C, F>(self, f: F) -> Result<T, ContextError<E>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| ContextError::new(e, f().to_string()))
    }

    #[inline]
    fn to_validation(self) -> Validation<T, E> {
        match self {
            Ok(value) => Validation::Success(value),
            Err(error) => Validation::Failure(error),
        }
    }

    #[inline]
    fn to_either(self) -> Either<E, T> {
        match self {
            Ok(value) => Either::Right(value),
            Err(error) => Either::Left(error),
        }
    }

    #[inline]
    fn swap(self) -> Result<E, T> {
        match self {
            Ok(value) => Err(value),
            Err(error) => Ok(error),
        }
    }

    #[inline]
    fn contains<U>(&self, value: &U) -> bool
    where
        T: PartialEq<U>,
    {
        match self {
            Ok(ref v) => v == value,
            Err(_) => false,
        }
    }

    #[inline]
    fn contains_err<U>(&self, value: &U) -> bool
    where
        E: PartialEq<U>,
    {
        match self {
            Ok(_) => false,
            Err(ref e) => e == value,
        }
    }
}
```

#### Flatten and Transpose Extensions

```rust
// Additional implementations for specific Result shapes

/// Extension for nested Results.
pub trait ResultFlattenExt<T, E> {
    /// Flatten a nested Result.
    fn flatten(self) -> Result<T, E>;
}

impl<T, E> ResultFlattenExt<T, E> for Result<Result<T, E>, E> {
    #[inline]
    fn flatten(self) -> Result<T, E> {
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e),
        }
    }
}

/// Extension for Result<Option<T>, E>.
pub trait ResultTransposeExt<T, E> {
    /// Transpose Result<Option<T>, E> to Option<Result<T, E>>.
    fn transpose(self) -> Option<Result<T, E>>;
}

impl<T, E> ResultTransposeExt<T, E> for Result<Option<T>, E> {
    #[inline]
    fn transpose(self) -> Option<Result<T, E>> {
        match self {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
```

#### Effect Conversion Extension

```rust
// In src/effect/from_result.rs or result_ext.rs

use crate::effect::{Effect, Pure, Fail};

/// Extension for converting Result to Effect.
pub trait ResultToEffectExt<T, E> {
    /// Convert this Result to an Effect.
    ///
    /// Ok becomes a pure effect, Err becomes a failed effect.
    fn to_effect<Env>(self) -> impl Effect<Output = T, Error = E, Env = Env>
    where
        T: Send + 'static,
        E: Send + 'static,
        Env: Sync + 'static;
}

impl<T, E> ResultToEffectExt<T, E> for Result<T, E> {
    fn to_effect<Env>(self) -> impl Effect<Output = T, Error = E, Env = Env>
    where
        T: Send + 'static,
        E: Send + 'static,
        Env: Sync + 'static,
    {
        crate::effect::from_result(self)
    }
}
```

### Module Structure

```
src/
├── lib.rs              # Add: pub mod result_ext; pub use result_ext::ResultExt;
├── result_ext.rs       # Main extension trait
```

### Prelude Integration

```rust
// In src/prelude.rs (if exists) or lib.rs

pub use crate::result_ext::{
    ResultExt,
    ResultFlattenExt,
    ResultTransposeExt,
    ResultToEffectExt,
};
```

## Dependencies

### Prerequisites
- Spec 026 (Either) - for `to_either` method
- Existing `ContextError` type
- Existing `Validation` type

### Affected Components
- None directly (extension trait)

### External Dependencies
- None

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // tap tests
    #[test]
    fn test_tap_ok_on_ok() {
        let mut called = false;
        let result: Result<i32, &str> = Ok(42);
        let returned = result.tap_ok(|_| called = true);
        assert!(called);
        assert_eq!(returned, Ok(42));
    }

    #[test]
    fn test_tap_ok_on_err() {
        let mut called = false;
        let result: Result<i32, &str> = Err("error");
        let returned = result.tap_ok(|_| called = true);
        assert!(!called);
        assert_eq!(returned, Err("error"));
    }

    #[test]
    fn test_tap_err_on_err() {
        let mut called = false;
        let result: Result<i32, &str> = Err("error");
        let returned = result.tap_err(|_| called = true);
        assert!(called);
        assert_eq!(returned, Err("error"));
    }

    #[test]
    fn test_tap_err_on_ok() {
        let mut called = false;
        let result: Result<i32, &str> = Ok(42);
        let returned = result.tap_err(|_| called = true);
        assert!(!called);
        assert_eq!(returned, Ok(42));
    }

    // ensure tests
    #[test]
    fn test_ensure_passes() {
        let result: Result<i32, &str> = Ok(42);
        let checked = result.ensure(|x| *x > 0, "negative");
        assert_eq!(checked, Ok(42));
    }

    #[test]
    fn test_ensure_fails() {
        let result: Result<i32, &str> = Ok(-5);
        let checked = result.ensure(|x| *x > 0, "negative");
        assert_eq!(checked, Err("negative"));
    }

    #[test]
    fn test_ensure_passes_err_through() {
        let result: Result<i32, &str> = Err("prior");
        let checked = result.ensure(|_| false, "other");
        assert_eq!(checked, Err("prior"));
    }

    #[test]
    fn test_ensure_with() {
        let result: Result<i32, String> = Ok(-5);
        let checked = result.ensure_with(
            |x| *x > 0,
            |x| format!("{} is not positive", x)
        );
        assert_eq!(checked, Err("-5 is not positive".to_string()));
    }

    #[test]
    fn test_unless_passes() {
        let result: Result<i32, &str> = Ok(5);
        let checked = result.unless(|x| *x < 0, "is negative");
        assert_eq!(checked, Ok(5));
    }

    #[test]
    fn test_unless_fails() {
        let result: Result<i32, &str> = Ok(-5);
        let checked = result.unless(|x| *x < 0, "is negative");
        assert_eq!(checked, Err("is negative"));
    }

    // context tests
    #[test]
    fn test_context_on_err() {
        let result: Result<i32, &str> = Err("io error");
        let with_context = result.context("reading file");
        assert!(with_context.is_err());
        let err = with_context.unwrap_err();
        assert!(err.to_string().contains("reading file"));
    }

    #[test]
    fn test_context_on_ok() {
        let result: Result<i32, &str> = Ok(42);
        let with_context = result.context("operation");
        assert_eq!(with_context.map_err(|_| ()), Ok(42));
    }

    // conversion tests
    #[test]
    fn test_to_validation_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(result.to_validation(), Validation::Success(42));
    }

    #[test]
    fn test_to_validation_err() {
        let result: Result<i32, &str> = Err("error");
        assert_eq!(result.to_validation(), Validation::Failure("error"));
    }

    #[test]
    fn test_to_either_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(result.to_either(), Either::Right(42));
    }

    #[test]
    fn test_to_either_err() {
        let result: Result<i32, &str> = Err("error");
        assert_eq!(result.to_either(), Either::Left("error"));
    }

    #[test]
    fn test_swap() {
        let ok: Result<i32, &str> = Ok(42);
        assert_eq!(ok.swap(), Err(42));

        let err: Result<i32, &str> = Err("error");
        assert_eq!(err.swap(), Ok("error"));
    }

    // query tests
    #[test]
    fn test_contains() {
        let result: Result<i32, &str> = Ok(42);
        assert!(result.contains(&42));
        assert!(!result.contains(&0));

        let err: Result<i32, &str> = Err("error");
        assert!(!err.contains(&42));
    }

    #[test]
    fn test_contains_err() {
        let result: Result<i32, &str> = Err("error");
        assert!(result.contains_err(&"error"));
        assert!(!result.contains_err(&"other"));

        let ok: Result<i32, &str> = Ok(42);
        assert!(!ok.contains_err(&"error"));
    }

    // flatten tests
    #[test]
    fn test_flatten_ok_ok() {
        let nested: Result<Result<i32, &str>, &str> = Ok(Ok(42));
        assert_eq!(nested.flatten(), Ok(42));
    }

    #[test]
    fn test_flatten_ok_err() {
        let nested: Result<Result<i32, &str>, &str> = Ok(Err("inner"));
        assert_eq!(nested.flatten(), Err("inner"));
    }

    #[test]
    fn test_flatten_err() {
        let nested: Result<Result<i32, &str>, &str> = Err("outer");
        assert_eq!(nested.flatten(), Err("outer"));
    }

    // transpose tests
    #[test]
    fn test_transpose() {
        let ok_some: Result<Option<i32>, &str> = Ok(Some(42));
        assert_eq!(ok_some.transpose(), Some(Ok(42)));

        let ok_none: Result<Option<i32>, &str> = Ok(None);
        assert_eq!(ok_none.transpose(), None);

        let err: Result<Option<i32>, &str> = Err("error");
        assert_eq!(err.transpose(), Some(Err("error")));
    }

    // chaining tests
    #[test]
    fn test_chaining() {
        let result: Result<i32, String> = Ok(42);
        let processed = result
            .tap_ok(|_| ())
            .ensure(|x| *x > 0, "negative".to_string())
            .ensure(|x| *x < 100, "too large".to_string());
        assert_eq!(processed, Ok(42));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_with_validation_roundtrip() {
        let result: Result<i32, &str> = Ok(42);
        let validation = result.to_validation();
        let back: Result<i32, &str> = validation.into();
        assert_eq!(back, Ok(42));
    }

    #[tokio::test]
    async fn test_to_effect() {
        let result: Result<i32, String> = Ok(42);
        let effect = result.to_effect::<()>();
        assert_eq!(effect.execute(&()).await, Ok(42));
    }
}
```

## Documentation Requirements

### Code Documentation

```rust
/// Extension trait for `std::result::Result` with functional programming combinators.
///
/// This trait provides additional methods on `Result` that integrate well with
/// Stillwater's functional programming style while being useful standalone.
///
/// # Side Effects with tap
///
/// Use `tap_ok` and `tap_err` to perform side effects (like logging) without
/// consuming the Result:
///
/// ```rust
/// use stillwater::ResultExt;
///
/// let result = fetch_data()
///     .tap_ok(|data| log::info!("Got {} bytes", data.len()))
///     .tap_err(|e| log::error!("Failed: {}", e));
/// ```
///
/// # Validation with ensure
///
/// Use `ensure` to add validation to Result chains:
///
/// ```rust
/// use stillwater::ResultExt;
///
/// let result = parse_age(input)
///     .ensure(|age| *age >= 0, "age cannot be negative")
///     .ensure(|age| *age <= 150, "age too large");
/// ```
///
/// # Error Context
///
/// Use `context` to add context to errors for better debugging:
///
/// ```rust
/// use stillwater::ResultExt;
///
/// fn load_config() -> Result<Config, ConfigError> {
///     read_file(path)
///         .context("reading config file")?
///         .parse()
///         .context("parsing config")
/// }
/// ```
///
/// # Conversions
///
/// Convert between Stillwater types:
///
/// ```rust
/// use stillwater::ResultExt;
///
/// let result: Result<i32, &str> = Ok(42);
///
/// // To Validation for error accumulation
/// let validation = result.to_validation();
///
/// // To Either for neutral sum type
/// let either = result.to_either();
///
/// // To Effect for effect composition
/// let effect = result.to_effect::<()>();
/// ```
```

### User Guide Section

```markdown
## Working with Result

Stillwater provides a `ResultExt` trait that adds useful combinators to `std::result::Result`.

### Import

```rust
use stillwater::ResultExt;
```

### Side Effects (Logging, Metrics)

```rust
let user = fetch_user(id)
    .tap_ok(|u| metrics::record_user_fetch())
    .tap_err(|e| log::error!("User fetch failed: {}", e));
```

### Inline Validation

```rust
let validated = result
    .ensure(|user| user.age >= 18, Error::TooYoung)
    .ensure(|user| !user.is_banned, Error::Banned);
```

### Error Context

```rust
let config = read_file(path)
    .context("reading configuration")?
    .parse()
    .context("parsing YAML")?;

// Error output:
// Error: parsing YAML
//   -> reading configuration
//   -> file not found
```
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Extension trait | Can't modify std types; extension is idiomatic |
| `tap_ok` not `tap` | Avoids confusion with std `.map` |
| `ensure` matches Effect | Consistent API across Stillwater |
| `context` uses Stillwater type | Integration with existing error handling |

### Naming Conventions

- `tap_*`: Side effects, borrowed value, returns self
- `ensure/unless`: Validation with error
- `to_*`: Conversion to other types
- `contains*`: Query methods

### Future Enhancements

1. **`tap_async`**: Async side effects
2. **`ensure_async`**: Async predicates
3. **`bimap`**: Transform both Ok and Err
4. **`recover`**: Error recovery combinators

## Migration and Compatibility

- **Breaking changes**: None (extension trait, no conflicts with std)
- **Feature flag**: None needed (always available)

---

*"Make Result work harder for you."*
