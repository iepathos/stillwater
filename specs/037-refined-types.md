---
number: 37
title: Refined Types
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-12-20
---

# Specification 037: Refined Types

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None (integrates with existing Validation type)

## Context

The "parse, don't validate" pattern is a fundamental functional programming principle: validate data once at system boundaries, then use types to guarantee validity throughout the codebase. This eliminates redundant runtime checks and makes invalid states unrepresentable.

Currently, Rust developers either:
1. Scatter validation checks throughout code (error-prone, verbose)
2. Use raw types and hope invariants hold (unsafe, bugs)
3. Write custom newtype wrappers manually (tedious, inconsistent)

Stillwater already provides `Validation<T, E>` for error accumulation and `NonEmptyVec<T>` as a refined collection type. This specification generalizes the pattern with a composable refined types system.

### The Problem

```rust
// Scattered validation - checks everywhere, easy to forget
fn process_user(name: String, age: i32, email: String) -> Result<User, Error> {
    if name.is_empty() {
        return Err(Error::EmptyName);
    }
    if age <= 0 {
        return Err(Error::InvalidAge);
    }
    if !email.contains('@') {
        return Err(Error::InvalidEmail);
    }
    // Finally do something...
    // But what if we call another function that needs these checks?
    // Do we check again? Trust the caller?
}
```

### The Solution

```rust
// Types encode the invariants - validated once, trusted everywhere
fn process_user(name: NonEmptyString, age: PositiveInt, email: Email) -> User {
    // name is GUARANTEED non-empty by construction
    // age is GUARANTEED positive by construction
    // email is GUARANTEED valid by construction
    // No runtime checks needed here!
}

// Validation happens at the boundary
fn handle_request(input: RawInput) -> Validation<User, Vec<ValidationError>> {
    Validation::all((
        NonEmptyString::validate(input.name),
        PositiveInt::validate(input.age),
        Email::validate(input.email),
    ))
    .map(|(name, age, email)| process_user(name, age, email))
}
```

## Objective

Provide a composable refined types system that:

1. Enables "parse, don't validate" pattern
2. Integrates with stillwater's `Validation` for error accumulation
3. Provides common predicates as building blocks
4. Allows predicate composition (`And`, `Or`, `Not`)
5. Has zero runtime overhead for refined value access
6. Requires no macros - pure trait-based implementation

## Requirements

### Functional Requirements

#### 1. Predicate Trait

- Define `Predicate<T>` trait for refinement conditions
- Associated `Error` type for predicate-specific errors
- `check(value: &T) -> Result<(), Self::Error>` method
- Optional `description() -> &'static str` for error messages
- Predicates must be `Send + Sync` for use across threads

#### 2. Refined Wrapper Type

- `Refined<T, P: Predicate<T>>` wraps value with predicate guarantee
- `new(value: T) -> Result<Self, P::Error>` validates on construction
- `get(&self) -> &T` for zero-cost access to inner value
- `into_inner(self) -> T` consumes and returns inner value
- `new_unchecked(value: T) -> Self` for trusted contexts (e.g., deserialization with known-valid data)
- Implement `AsRef<T>`, `Deref`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` where inner type supports them

#### 3. Common Numeric Predicates

- `Positive` - value > 0 (for i32, i64, f64, etc.)
- `NonNegative` - value >= 0
- `Negative` - value < 0
- `NonZero` - value != 0

#### 4. Common String Predicates

- `NonEmpty` - string is not empty (also works for Vec, collections)
- `Trimmed` - string equals its trimmed form (no leading/trailing whitespace)
- `MaxLength<const N: usize>` - string length <= N
- `MinLength<const N: usize>` - string length >= N

#### 5. Common Collection Predicates

- `NonEmpty` - collection is not empty (shared with String)
- `MaxSize<const N: usize>` - collection size <= N
- `MinSize<const N: usize>` - collection size >= N

#### 6. Predicate Combinators

- `And<A, B>` - both predicates must hold
- `Or<A, B>` - at least one predicate must hold
- `Not<A>` - predicate must not hold

#### 7. Validation Integration

- `Refined::validate(value: T) -> Validation<Self, P::Error>` for single error
- `Refined::validate_collecting(value: T) -> Validation<Self, Vec<P::Error>>` for accumulation
- Seamless use with `Validation::all()` for multi-field validation

#### 8. Type Aliases for Ergonomics

Provide common refined types as type aliases:
- `type NonEmptyString = Refined<String, NonEmpty>`
- `type PositiveI32 = Refined<i32, Positive>`
- `type PositiveI64 = Refined<i64, Positive>`
- `type NonNegativeI32 = Refined<i32, NonNegative>`
- etc.

### Non-Functional Requirements

- **Zero runtime overhead**: `Refined<T, P>` has same memory layout as `T`
- **No macros**: Pure trait-based implementation, no proc-macros
- **Composable**: Predicates combine cleanly via `And`, `Or`, `Not`
- **Ergonomic**: Type aliases and method chaining for common cases
- **Thread-safe**: All types are `Send + Sync` where `T` is
- **Extensible**: Users can easily define custom predicates

## Acceptance Criteria

- [ ] `Predicate<T>` trait defined with `check`, `Error`, and `description`
- [ ] `Refined<T, P>` wrapper type with `new`, `get`, `into_inner`, `new_unchecked`
- [ ] `Refined` implements `AsRef`, `Deref`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
- [ ] `Positive` predicate implemented for `i32`, `i64`, `i128`, `f32`, `f64`
- [ ] `NonNegative` predicate implemented for `i32`, `i64`, `i128`, `f32`, `f64`
- [ ] `Negative` predicate implemented for `i32`, `i64`, `i128`, `f32`, `f64`
- [ ] `NonZero` predicate implemented for `i32`, `i64`, `i128`
- [ ] `NonEmpty` predicate implemented for `String`, `&str`, `Vec<T>`
- [ ] `Trimmed` predicate implemented for `String`
- [ ] `MaxLength<N>` predicate implemented for `String`
- [ ] `MinLength<N>` predicate implemented for `String`
- [ ] `MaxSize<N>` predicate implemented for `Vec<T>`
- [ ] `MinSize<N>` predicate implemented for `Vec<T>`
- [ ] `And<A, B>` combinator correctly checks both predicates
- [ ] `Or<A, B>` combinator correctly checks at least one predicate
- [ ] `Not<A>` combinator correctly inverts predicate
- [ ] `Refined::validate` returns `Validation::Success` or `Validation::Failure`
- [ ] `Refined::validate_collecting` returns errors in `Vec` for accumulation
- [ ] Type aliases provided for common combinations
- [ ] Comprehensive unit tests for all predicates
- [ ] Documentation with examples for defining custom predicates
- [ ] Integration tests showing use with `Validation::all`

## Technical Details

### Module Structure

```
src/refined/
├── mod.rs           # Module exports, Refined type, Predicate trait
├── predicates/
│   ├── mod.rs       # Re-exports
│   ├── numeric.rs   # Positive, NonNegative, Negative, NonZero
│   ├── string.rs    # NonEmpty, Trimmed, MaxLength, MinLength
│   └── collection.rs # NonEmpty, MaxSize, MinSize (for Vec)
├── combinators.rs   # And, Or, Not
├── validation.rs    # Validation integration methods
└── aliases.rs       # Type aliases (NonEmptyString, PositiveI32, etc.)
```

### Core Types

```rust
// src/refined/mod.rs

use std::marker::PhantomData;

/// A predicate that constrains values of type T.
///
/// Predicates are stateless - they only define the check logic.
/// The actual values are stored in `Refined<T, P>`.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::Predicate;
///
/// pub struct Even;
///
/// impl Predicate<i32> for Even {
///     type Error = &'static str;
///
///     fn check(value: &i32) -> Result<(), Self::Error> {
///         if value % 2 == 0 {
///             Ok(())
///         } else {
///             Err("value must be even")
///         }
///     }
/// }
/// ```
pub trait Predicate<T>: Send + Sync + 'static {
    /// Error returned when the predicate fails
    type Error: Send + Sync;

    /// Check if the value satisfies the predicate
    fn check(value: &T) -> Result<(), Self::Error>;

    /// Human-readable description of what this predicate requires
    fn description() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A value of type T that is guaranteed to satisfy predicate P.
///
/// `Refined` provides the "parse, don't validate" pattern:
/// - Validate once at construction
/// - Access freely without runtime checks
/// - Types document invariants
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, NonEmpty};
///
/// type NonEmptyString = Refined<String, NonEmpty>;
///
/// fn greet(name: NonEmptyString) {
///     // name is guaranteed non-empty - no check needed!
///     println!("Hello, {}!", name.get());
/// }
///
/// // At the boundary, we validate
/// let name = NonEmptyString::new("Alice".to_string())?;
/// greet(name);
/// ```
#[derive(Debug)]
pub struct Refined<T, P: Predicate<T>> {
    value: T,
    _predicate: PhantomData<P>,
}

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Create a new refined value, checking the predicate.
    ///
    /// Returns `Ok(Refined)` if the predicate passes,
    /// `Err(P::Error)` if it fails.
    pub fn new(value: T) -> Result<Self, P::Error> {
        P::check(&value)?;
        Ok(Self { value, _predicate: PhantomData })
    }

    /// Get a reference to the inner value.
    ///
    /// This is zero-cost - no runtime check.
    #[inline]
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Consume the refined value, returning the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Create a refined value without checking the predicate.
    ///
    /// # Safety (Logical)
    ///
    /// This is not unsafe in the Rust memory sense, but it bypasses
    /// the predicate check. The caller must guarantee that the
    /// predicate would pass. Use this for:
    /// - Deserializing known-valid data
    /// - Performance-critical code where validity is proven elsewhere
    /// - Internal code after transformation that preserves invariants
    #[inline]
    pub fn new_unchecked(value: T) -> Self {
        Self { value, _predicate: PhantomData }
    }

    /// Map the inner value, re-checking the predicate.
    ///
    /// Returns `Err` if the new value doesn't satisfy the predicate.
    pub fn try_map<F>(self, f: F) -> Result<Self, P::Error>
    where
        F: FnOnce(T) -> T,
    {
        Self::new(f(self.value))
    }
}

// Clone when T: Clone
impl<T: Clone, P: Predicate<T>> Clone for Refined<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _predicate: PhantomData,
        }
    }
}

// PartialEq delegates to inner
impl<T: PartialEq, P: Predicate<T>> PartialEq for Refined<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, P: Predicate<T>> Eq for Refined<T, P> {}

// Hash delegates to inner
impl<T: std::hash::Hash, P: Predicate<T>> std::hash::Hash for Refined<T, P> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

// AsRef for transparent access
impl<T, P: Predicate<T>> AsRef<T> for Refined<T, P> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

// Deref for ergonomic access
impl<T, P: Predicate<T>> std::ops::Deref for Refined<T, P> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

// Display when T: Display
impl<T: std::fmt::Display, P: Predicate<T>> std::fmt::Display for Refined<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
```

### Numeric Predicates

```rust
// src/refined/predicates/numeric.rs

use super::Predicate;

/// Value must be positive (> 0)
pub struct Positive;

/// Value must be non-negative (>= 0)
pub struct NonNegative;

/// Value must be negative (< 0)
pub struct Negative;

/// Value must be non-zero (!= 0)
pub struct NonZero;

// Macro to reduce repetition for numeric implementations
macro_rules! impl_numeric_predicate {
    ($pred:ty, $check:expr, $msg:expr, $desc:expr, [$($ty:ty),+]) => {
        $(
            impl Predicate<$ty> for $pred {
                type Error = &'static str;

                fn check(value: &$ty) -> Result<(), Self::Error> {
                    if $check(*value) {
                        Ok(())
                    } else {
                        Err($msg)
                    }
                }

                fn description() -> &'static str {
                    $desc
                }
            }
        )+
    };
}

impl_numeric_predicate!(
    Positive,
    |v| v > 0,
    "value must be positive",
    "positive number (> 0)",
    [i8, i16, i32, i64, i128, isize]
);

impl_numeric_predicate!(
    Positive,
    |v: f32| v > 0.0,
    "value must be positive",
    "positive number (> 0)",
    [f32]
);

impl_numeric_predicate!(
    Positive,
    |v: f64| v > 0.0,
    "value must be positive",
    "positive number (> 0)",
    [f64]
);

impl_numeric_predicate!(
    NonNegative,
    |v| v >= 0,
    "value must be non-negative",
    "non-negative number (>= 0)",
    [i8, i16, i32, i64, i128, isize]
);

impl_numeric_predicate!(
    NonNegative,
    |v: f32| v >= 0.0,
    "value must be non-negative",
    "non-negative number (>= 0)",
    [f32]
);

impl_numeric_predicate!(
    NonNegative,
    |v: f64| v >= 0.0,
    "value must be non-negative",
    "non-negative number (>= 0)",
    [f64]
);

impl_numeric_predicate!(
    Negative,
    |v| v < 0,
    "value must be negative",
    "negative number (< 0)",
    [i8, i16, i32, i64, i128, isize]
);

impl_numeric_predicate!(
    Negative,
    |v: f32| v < 0.0,
    "value must be negative",
    "negative number (< 0)",
    [f32]
);

impl_numeric_predicate!(
    Negative,
    |v: f64| v < 0.0,
    "value must be negative",
    "negative number (< 0)",
    [f64]
);

impl_numeric_predicate!(
    NonZero,
    |v| v != 0,
    "value must be non-zero",
    "non-zero number (!= 0)",
    [i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize]
);
```

### String Predicates

```rust
// src/refined/predicates/string.rs

use super::Predicate;

/// String must not be empty
pub struct NonEmpty;

impl Predicate<String> for NonEmpty {
    type Error = &'static str;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err("string cannot be empty")
        } else {
            Ok(())
        }
    }

    fn description() -> &'static str {
        "non-empty string"
    }
}

impl Predicate<&str> for NonEmpty {
    type Error = &'static str;

    fn check(value: &&str) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err("string cannot be empty")
        } else {
            Ok(())
        }
    }
}

/// String equals its trimmed form (no leading/trailing whitespace)
pub struct Trimmed;

impl Predicate<String> for Trimmed {
    type Error = &'static str;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.trim() == value {
            Ok(())
        } else {
            Err("string has leading or trailing whitespace")
        }
    }

    fn description() -> &'static str {
        "trimmed string (no leading/trailing whitespace)"
    }
}

/// String length must be at most N
pub struct MaxLength<const N: usize>;

impl<const N: usize> Predicate<String> for MaxLength<N> {
    type Error = String;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.len() <= N {
            Ok(())
        } else {
            Err(format!("string length {} exceeds maximum {}", value.len(), N))
        }
    }

    fn description() -> &'static str {
        // Can't include N in static str, use type name
        "string with maximum length"
    }
}

/// String length must be at least N
pub struct MinLength<const N: usize>;

impl<const N: usize> Predicate<String> for MinLength<N> {
    type Error = String;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.len() >= N {
            Ok(())
        } else {
            Err(format!("string length {} is less than minimum {}", value.len(), N))
        }
    }

    fn description() -> &'static str {
        "string with minimum length"
    }
}
```

### Collection Predicates

```rust
// src/refined/predicates/collection.rs

use super::Predicate;

// NonEmpty is defined in string.rs but also works for Vec

impl<T> Predicate<Vec<T>> for super::string::NonEmpty {
    type Error = &'static str;

    fn check(value: &Vec<T>) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err("collection cannot be empty")
        } else {
            Ok(())
        }
    }

    fn description() -> &'static str {
        "non-empty collection"
    }
}

/// Collection size must be at most N
pub struct MaxSize<const N: usize>;

impl<const N: usize, T> Predicate<Vec<T>> for MaxSize<N> {
    type Error = String;

    fn check(value: &Vec<T>) -> Result<(), Self::Error> {
        if value.len() <= N {
            Ok(())
        } else {
            Err(format!("collection size {} exceeds maximum {}", value.len(), N))
        }
    }
}

/// Collection size must be at least N
pub struct MinSize<const N: usize>;

impl<const N: usize, T> Predicate<Vec<T>> for MinSize<N> {
    type Error = String;

    fn check(value: &Vec<T>) -> Result<(), Self::Error> {
        if value.len() >= N {
            Ok(())
        } else {
            Err(format!("collection size {} is less than minimum {}", value.len(), N))
        }
    }
}
```

### Combinators

```rust
// src/refined/combinators.rs

use super::Predicate;
use std::marker::PhantomData;

/// Both predicates must hold
pub struct And<A, B>(PhantomData<(A, B)>);

impl<T, A, B> Predicate<T> for And<A, B>
where
    A: Predicate<T>,
    B: Predicate<T>,
{
    type Error = AndError<A::Error, B::Error>;

    fn check(value: &T) -> Result<(), Self::Error> {
        let a_result = A::check(value);
        let b_result = B::check(value);

        match (a_result, b_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(a), Ok(())) => Err(AndError::First(a)),
            (Ok(()), Err(b)) => Err(AndError::Second(b)),
            (Err(a), Err(b)) => Err(AndError::Both(a, b)),
        }
    }

    fn description() -> &'static str {
        "both predicates must hold"
    }
}

/// Error type for And combinator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndError<A, B> {
    First(A),
    Second(B),
    Both(A, B),
}

impl<A: std::fmt::Display, B: std::fmt::Display> std::fmt::Display for AndError<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AndError::First(a) => write!(f, "{}", a),
            AndError::Second(b) => write!(f, "{}", b),
            AndError::Both(a, b) => write!(f, "{}; {}", a, b),
        }
    }
}

impl<A: std::error::Error + 'static, B: std::error::Error + 'static> std::error::Error for AndError<A, B> {}

/// At least one predicate must hold
pub struct Or<A, B>(PhantomData<(A, B)>);

impl<T, A, B> Predicate<T> for Or<A, B>
where
    A: Predicate<T>,
    B: Predicate<T>,
{
    type Error = OrError<A::Error, B::Error>;

    fn check(value: &T) -> Result<(), Self::Error> {
        match A::check(value) {
            Ok(()) => Ok(()),
            Err(a_err) => match B::check(value) {
                Ok(()) => Ok(()),
                Err(b_err) => Err(OrError(a_err, b_err)),
            },
        }
    }

    fn description() -> &'static str {
        "at least one predicate must hold"
    }
}

/// Error type for Or combinator (both predicates failed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrError<A, B>(pub A, pub B);

impl<A: std::fmt::Display, B: std::fmt::Display> std::fmt::Display for OrError<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "neither predicate held: {} and {}", self.0, self.1)
    }
}

impl<A: std::error::Error + 'static, B: std::error::Error + 'static> std::error::Error for OrError<A, B> {}

/// Predicate must NOT hold
pub struct Not<A>(PhantomData<A>);

impl<T, A: Predicate<T>> Predicate<T> for Not<A> {
    type Error = NotError;

    fn check(value: &T) -> Result<(), Self::Error> {
        match A::check(value) {
            Ok(()) => Err(NotError(A::description())),
            Err(_) => Ok(()),
        }
    }

    fn description() -> &'static str {
        "predicate must not hold"
    }
}

/// Error type for Not combinator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotError(pub &'static str);

impl std::fmt::Display for NotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value must NOT satisfy: {}", self.0)
    }
}

impl std::error::Error for NotError {}
```

### Validation Integration

```rust
// src/refined/validation.rs

use super::{Predicate, Refined};
use crate::Validation;

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Validate a value, returning a Validation result.
    ///
    /// Use this with `Validation::all` for error accumulation.
    pub fn validate(value: T) -> Validation<Self, P::Error> {
        match Self::new(value) {
            Ok(refined) => Validation::Success(refined),
            Err(e) => Validation::Failure(e),
        }
    }
}

impl<T, P> Refined<T, P>
where
    P: Predicate<T>,
{
    /// Validate a value, wrapping the error in a Vec for accumulation.
    ///
    /// This is useful when combining with other validations that
    /// produce `Vec<E>` errors.
    pub fn validate_vec(value: T) -> Validation<Self, Vec<P::Error>> {
        match Self::new(value) {
            Ok(refined) => Validation::Success(refined),
            Err(e) => Validation::Failure(vec![e]),
        }
    }
}

/// Extension trait for creating refined validations with field context
pub trait RefinedValidationExt<T, P: Predicate<T>> {
    /// Validate with a field name for error context
    fn validate_field(value: T, field: &'static str) -> Validation<Refined<T, P>, FieldError<P::Error>>;
}

/// Error with field context
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError<E> {
    pub field: &'static str,
    pub error: E,
}

impl<E: std::fmt::Display> std::fmt::Display for FieldError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.error)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for FieldError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl<T, P: Predicate<T>> RefinedValidationExt<T, P> for Refined<T, P> {
    fn validate_field(value: T, field: &'static str) -> Validation<Refined<T, P>, FieldError<P::Error>> {
        match Refined::new(value) {
            Ok(refined) => Validation::Success(refined),
            Err(e) => Validation::Failure(FieldError { field, error: e }),
        }
    }
}
```

### Type Aliases

```rust
// src/refined/aliases.rs

use super::{Refined, NonEmpty, Positive, NonNegative, NonZero, Trimmed};
use super::combinators::And;

// String aliases
pub type NonEmptyString = Refined<String, NonEmpty>;
pub type TrimmedString = Refined<String, Trimmed>;
pub type NonEmptyTrimmedString = Refined<String, And<NonEmpty, Trimmed>>;

// Signed integer aliases
pub type PositiveI8 = Refined<i8, Positive>;
pub type PositiveI16 = Refined<i16, Positive>;
pub type PositiveI32 = Refined<i32, Positive>;
pub type PositiveI64 = Refined<i64, Positive>;
pub type PositiveI128 = Refined<i128, Positive>;
pub type PositiveIsize = Refined<isize, Positive>;

pub type NonNegativeI8 = Refined<i8, NonNegative>;
pub type NonNegativeI16 = Refined<i16, NonNegative>;
pub type NonNegativeI32 = Refined<i32, NonNegative>;
pub type NonNegativeI64 = Refined<i64, NonNegative>;
pub type NonNegativeI128 = Refined<i128, NonNegative>;
pub type NonNegativeIsize = Refined<isize, NonNegative>;

pub type NonZeroI8 = Refined<i8, NonZero>;
pub type NonZeroI16 = Refined<i16, NonZero>;
pub type NonZeroI32 = Refined<i32, NonZero>;
pub type NonZeroI64 = Refined<i64, NonZero>;
pub type NonZeroI128 = Refined<i128, NonZero>;
pub type NonZeroIsize = Refined<isize, NonZero>;

// Unsigned integer aliases (NonZero only, since always non-negative and > 0 when non-zero)
pub type NonZeroU8 = Refined<u8, NonZero>;
pub type NonZeroU16 = Refined<u16, NonZero>;
pub type NonZeroU32 = Refined<u32, NonZero>;
pub type NonZeroU64 = Refined<u64, NonZero>;
pub type NonZeroU128 = Refined<u128, NonZero>;
pub type NonZeroUsize = Refined<usize, NonZero>;

// Float aliases
pub type PositiveF32 = Refined<f32, Positive>;
pub type PositiveF64 = Refined<f64, Positive>;
pub type NonNegativeF32 = Refined<f32, NonNegative>;
pub type NonNegativeF64 = Refined<f64, NonNegative>;

// Collection aliases
pub type NonEmptyVec<T> = Refined<Vec<T>, NonEmpty>;
```

### Usage Examples

```rust
use stillwater::refined::{Refined, NonEmpty, Positive, And, Trimmed};
use stillwater::refined::aliases::*;
use stillwater::Validation;

// Simple usage
fn greet(name: NonEmptyString) {
    println!("Hello, {}!", name.get());
}

let name = NonEmptyString::new("Alice".to_string()).unwrap();
greet(name);

// Custom predicate
pub struct ValidEmail;

impl Predicate<String> for ValidEmail {
    type Error = &'static str;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.contains('@') && value.contains('.') {
            Ok(())
        } else {
            Err("invalid email format")
        }
    }
}

type Email = Refined<String, ValidEmail>;

// Combined predicates
type Username = Refined<String, And<NonEmpty, Trimmed>>;

// Validation with error accumulation
struct UserInput {
    name: String,
    age: i32,
    email: String,
}

struct ValidUser {
    name: NonEmptyString,
    age: PositiveI32,
    email: Email,
}

fn validate_user(input: UserInput) -> Validation<ValidUser, Vec<&'static str>> {
    Validation::all((
        NonEmptyString::validate_vec(input.name),
        PositiveI32::validate_vec(input.age),
        Email::validate_vec(input.email),
    ))
    .map(|(name, age, email)| ValidUser { name, age, email })
}

// All errors collected
let result = validate_user(UserInput {
    name: "".to_string(),   // Error: empty
    age: -5,                 // Error: not positive
    email: "invalid".to_string(), // Error: no @ or .
});
// Failure(["string cannot be empty", "value must be positive", "invalid email format"])
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**:
  - `src/lib.rs` - add `refined` module export
  - `src/prelude.rs` - optionally add common refined types
- **External Dependencies**: None (uses only std library)

## Testing Strategy

### Unit Tests

- Test each predicate with passing and failing values
- Test edge cases (empty string, zero, boundary values)
- Test all numeric types for each numeric predicate
- Test combinator logic (And, Or, Not) with various predicate combinations
- Test Refined methods: new, get, into_inner, new_unchecked
- Test trait implementations: Clone, PartialEq, Hash, Display

### Integration Tests

- Test with Validation::all for multi-field validation
- Test complex predicate compositions
- Test error accumulation patterns
- Test custom predicate definitions

### Property Tests

- Property: `Refined::new(x).map(|r| r.into_inner()) == Ok(x)` when predicate passes
- Property: `Refined::get` always returns value satisfying predicate
- Property: `And<A, B>` passes iff both A and B pass
- Property: `Or<A, B>` passes iff at least one passes
- Property: `Not<A>` passes iff A fails

## Documentation Requirements

### Code Documentation

- Doc comments on all public types and methods
- Examples in doc comments for common patterns
- Explanation of "parse, don't validate" philosophy

### User Documentation

- Add "Refined Types" section to README.md
- Tutorial showing progression from raw types to refined types
- Examples of custom predicate definition
- Common patterns: validation at boundaries, domain modeling

## Implementation Notes

### Zero-Cost Abstraction

`Refined<T, P>` must have the same memory layout as `T`:
- `PhantomData<P>` is zero-sized
- No runtime predicate storage
- `get()` and `Deref` are inlined

### Predicate Statelessness

Predicates are stateless marker types:
- No instance data
- All methods are static
- Enables zero-cost composition

### Error Type Design

Predicate errors should be informative:
- Use `&'static str` for simple cases
- Use `String` when dynamic info needed (e.g., length limits)
- Consider structured error types for complex validation

### Relationship to NonEmptyVec

The existing `NonEmptyVec<T>` in stillwater is conceptually a refined type. Consider:
- Keeping both for backwards compatibility
- Making `NonEmptyVec` an alias for `Refined<Vec<T>, NonEmpty>`
- Or keeping separate for different construction APIs

## Migration and Compatibility

### Backward Compatibility

This is purely additive - no breaking changes.

### Migration Path

1. **No change required**: Existing code works as-is
2. **Gradual adoption**: Replace raw types with refined types at boundaries
3. **Full adoption**: Use refined types in function signatures throughout

### Future: Serde Integration

A future spec could add serde support:
```rust
#[derive(Serialize, Deserialize)]
#[serde(try_from = "String")]
struct Email(Refined<String, ValidEmail>);
```

This would enable parsing and validation during deserialization.
