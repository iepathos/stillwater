---
number: 028
title: Predicate Combinators Module
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 028: Predicate Combinators Module

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

### The Problem

Validation logic often involves multiple predicates that need to be combined. Currently, users must write verbose boolean expressions or create ad-hoc helper functions:

```rust
// Current: Verbose, hard to reuse
fn validate_username(name: &str) -> Validation<&str, Vec<String>> {
    if !name.is_empty()
        && name.len() >= 3
        && name.len() <= 20
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        Validation::success(name)
    } else {
        Validation::failure(vec!["Invalid username".to_string()])
    }
}

// Can't easily reuse "length between 3 and 20" elsewhere
// Can't compose predicates from a library
```

### The Solution

Predicate combinators allow building complex predicates from simple, reusable pieces:

```rust
// With predicate combinators: Composable, reusable, clear
use stillwater::predicate::*;

let valid_username = not_empty()
    .and(len_between(3, 20))
    .and(all_chars(|c| c.is_alphanumeric() || c == '_'));

fn validate_username(name: &str) -> Validation<&str, Vec<String>> {
    if valid_username.check(name) {
        Validation::success(name)
    } else {
        Validation::failure(vec!["Invalid username".to_string()])
    }
}

// Reuse the length predicate elsewhere
let valid_password = len_between(8, 128)
    .and(contains_char(|c| c.is_uppercase()))
    .and(contains_char(|c| c.is_numeric()));
```

### Prior Art

- **Java**: `Predicate<T>` with `and()`, `or()`, `negate()`
- **Scala**: Predicate composition in validation libraries
- **Haskell**: Predicate combinators in various libraries
- **Rust**: `Iterator` predicates (`all`, `any`), but no standalone combinator library

## Objective

Add a `predicate` module to Stillwater that provides composable predicate combinators for use in validation pipelines, with first-class integration with `Validation` and `Effect`.

## Requirements

### Functional Requirements

#### FR1: Predicate Trait

- **MUST** define `Predicate<T>` trait for composable predicates
- **MUST** have `check(&self, value: &T) -> bool` method
- **MUST** be object-safe for dynamic dispatch when needed
- **SHOULD** have blanket impl for `Fn(&T) -> bool`

```rust
pub trait Predicate<T: ?Sized>: Send + Sync {
    fn check(&self, value: &T) -> bool;
}
```

#### FR2: Logical Combinators

- **MUST** provide `and(p1, p2)` combinator
- **MUST** provide `or(p1, p2)` combinator
- **MUST** provide `not(p)` combinator
- **MUST** provide `all_of(predicates)` for multiple AND
- **MUST** provide `any_of(predicates)` for multiple OR
- **MUST** provide `none_of(predicates)` (equivalent to `not(any_of(...))`)

#### FR3: Extension Trait for Chaining

- **MUST** provide `PredicateExt` trait with method chaining
- **MUST** have `.and(other)`, `.or(other)`, `.not()` methods
- **MUST** return concrete types (zero-cost)

```rust
pub trait PredicateExt<T: ?Sized>: Predicate<T> + Sized {
    fn and<P: Predicate<T>>(self, other: P) -> And<Self, P>;
    fn or<P: Predicate<T>>(self, other: P) -> Or<Self, P>;
    fn not(self) -> Not<Self>;
}
```

#### FR4: Common Predicates for Strings

- **MUST** provide `not_empty()` - string is not empty
- **MUST** provide `len_eq(n)` - exact length
- **MUST** provide `len_min(n)` - minimum length
- **MUST** provide `len_max(n)` - maximum length
- **MUST** provide `len_between(min, max)` - length in range (inclusive)
- **MUST** provide `starts_with(prefix)`
- **MUST** provide `ends_with(suffix)`
- **MUST** provide `contains(substring)`
- **MUST** provide `matches(regex)` - regex match (behind feature flag)
- **MUST** provide `all_chars(predicate)` - all characters satisfy predicate
- **MUST** provide `any_char(predicate)` - any character satisfies predicate
- **SHOULD** provide `is_ascii()`, `is_alphanumeric()`, etc.

#### FR5: Common Predicates for Numbers

- **MUST** provide `eq(value)` - equality
- **MUST** provide `ne(value)` - not equal
- **MUST** provide `gt(value)` - greater than
- **MUST** provide `ge(value)` - greater than or equal
- **MUST** provide `lt(value)` - less than
- **MUST** provide `le(value)` - less than or equal
- **MUST** provide `between(min, max)` - in range (inclusive)
- **MUST** provide `positive()` - greater than zero
- **MUST** provide `negative()` - less than zero
- **MUST** provide `non_negative()` - greater than or equal to zero

#### FR6: Common Predicates for Collections

- **MUST** provide `is_empty()` - collection is empty
- **MUST** provide `is_not_empty()` - collection has elements
- **MUST** provide `has_len(n)` - exact length
- **MUST** provide `has_min_len(n)` - minimum length
- **MUST** provide `has_max_len(n)` - maximum length
- **MUST** provide `all(predicate)` - all elements satisfy predicate
- **MUST** provide `any(predicate)` - any element satisfies predicate
- **MUST** provide `contains_element(value)` - contains specific value

#### FR7: Integration with Validation

- **MUST** provide `validate<T, E>(value: T, predicate: P, error: E) -> Validation<T, E>`
- **MUST** provide `Validation::ensure(predicate, error)` method
- **SHOULD** provide `validate_with_error<T, E, F>(value: T, predicate: P, error_fn: F)`

#### FR8: Integration with Effect

- **MUST** provide `Effect::filter_or(predicate, error)` method
- **SHOULD** integrate with Spec 029 (ensure/filter_or)

### Non-Functional Requirements

#### NFR1: Zero-Cost

- Combinator types MUST NOT allocate
- Predicate chains SHOULD compile to equivalent boolean expressions
- Size of combined predicates = sum of component sizes

#### NFR2: Ergonomics

- Method chaining SHOULD feel natural
- Type inference SHOULD work without annotations
- Common patterns SHOULD be one-liners

#### NFR3: Thread Safety

- All predicates MUST be `Send + Sync`
- Predicates MUST be reusable (no consumption)

## Acceptance Criteria

### Core Trait

- [ ] **AC1**: `Predicate<T>` trait compiles with `check` method
- [ ] **AC2**: Blanket impl works for closures: `|x: &i32| *x > 0`
- [ ] **AC3**: `PredicateExt` provides chaining methods

### Logical Combinators

- [ ] **AC4**: `and(p1, p2)` returns true only when both are true
- [ ] **AC5**: `or(p1, p2)` returns true when either is true
- [ ] **AC6**: `not(p)` inverts the result
- [ ] **AC7**: `all_of([p1, p2, p3])` works correctly
- [ ] **AC8**: `any_of([p1, p2, p3])` works correctly
- [ ] **AC9**: Chaining works: `p1.and(p2).or(p3).not()`

### String Predicates

- [ ] **AC10**: `not_empty().check("")` returns false
- [ ] **AC11**: `len_between(3, 10).check("hello")` returns true
- [ ] **AC12**: `starts_with("http").check("https://")` returns true
- [ ] **AC13**: `all_chars(char::is_alphabetic).check("abc")` returns true

### Number Predicates

- [ ] **AC14**: `positive().check(&5)` returns true
- [ ] **AC15**: `between(0, 100).check(&50)` returns true
- [ ] **AC16**: `gt(10).and(lt(20)).check(&15)` returns true

### Collection Predicates

- [ ] **AC17**: `is_empty().check(&vec![])` returns true
- [ ] **AC18**: `all(positive()).check(&vec![1, 2, 3])` returns true
- [ ] **AC19**: `contains_element(5).check(&vec![1, 5, 10])` returns true

### Integration

- [ ] **AC20**: `validate(value, predicate, error)` returns correct Validation
- [ ] **AC21**: Works in Effect filter chains

### Zero-Cost

- [ ] **AC22**: `And<P1, P2>` size equals `P1 size + P2 size`
- [ ] **AC23**: Chained predicates compile to simple boolean expressions

## Technical Details

### Implementation Approach

#### Core Trait and Types

```rust
// src/predicate/mod.rs

/// A composable predicate over values of type T.
///
/// Predicates can be combined using logical operators:
/// - `and`: Both predicates must be true
/// - `or`: Either predicate must be true
/// - `not`: Inverts the predicate
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let is_valid_age = ge(0).and(le(150));
/// assert!(is_valid_age.check(&25));
/// assert!(!is_valid_age.check(&-5));
/// ```
pub trait Predicate<T: ?Sized>: Send + Sync {
    /// Check if the value satisfies this predicate.
    fn check(&self, value: &T) -> bool;
}

// Blanket impl for closures
impl<T: ?Sized, F> Predicate<T> for F
where
    F: Fn(&T) -> bool + Send + Sync,
{
    fn check(&self, value: &T) -> bool {
        self(value)
    }
}

/// Extension trait for predicate combinators.
pub trait PredicateExt<T: ?Sized>: Predicate<T> + Sized {
    /// Combine with AND logic.
    fn and<P: Predicate<T>>(self, other: P) -> And<Self, P> {
        And(self, other)
    }

    /// Combine with OR logic.
    fn or<P: Predicate<T>>(self, other: P) -> Or<Self, P> {
        Or(self, other)
    }

    /// Invert the predicate.
    fn not(self) -> Not<Self> {
        Not(self)
    }
}

impl<T: ?Sized, P: Predicate<T>> PredicateExt<T> for P {}
```

#### Combinator Types

```rust
// src/predicate/combinators.rs

/// AND combinator - both predicates must be true.
#[derive(Clone, Copy)]
pub struct And<P1, P2>(pub P1, pub P2);

impl<T: ?Sized, P1: Predicate<T>, P2: Predicate<T>> Predicate<T> for And<P1, P2> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        self.0.check(value) && self.1.check(value)
    }
}

unsafe impl<P1: Send, P2: Send> Send for And<P1, P2> {}
unsafe impl<P1: Sync, P2: Sync> Sync for And<P1, P2> {}

/// OR combinator - either predicate must be true.
#[derive(Clone, Copy)]
pub struct Or<P1, P2>(pub P1, pub P2);

impl<T: ?Sized, P1: Predicate<T>, P2: Predicate<T>> Predicate<T> for Or<P1, P2> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        self.0.check(value) || self.1.check(value)
    }
}

unsafe impl<P1: Send, P2: Send> Send for Or<P1, P2> {}
unsafe impl<P1: Sync, P2: Sync> Sync for Or<P1, P2> {}

/// NOT combinator - inverts the predicate.
#[derive(Clone, Copy)]
pub struct Not<P>(pub P);

impl<T: ?Sized, P: Predicate<T>> Predicate<T> for Not<P> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        !self.0.check(value)
    }
}

unsafe impl<P: Send> Send for Not<P> {}
unsafe impl<P: Sync> Sync for Not<P> {}

/// Check if all predicates are satisfied.
pub fn all_of<T, I, P>(predicates: I) -> AllOf<P>
where
    I: IntoIterator<Item = P>,
    P: Predicate<T>,
{
    AllOf(predicates.into_iter().collect())
}

pub struct AllOf<P>(Vec<P>);

impl<T: ?Sized, P: Predicate<T>> Predicate<T> for AllOf<P> {
    fn check(&self, value: &T) -> bool {
        self.0.iter().all(|p| p.check(value))
    }
}

/// Check if any predicate is satisfied.
pub fn any_of<T, I, P>(predicates: I) -> AnyOf<P>
where
    I: IntoIterator<Item = P>,
    P: Predicate<T>,
{
    AnyOf(predicates.into_iter().collect())
}

pub struct AnyOf<P>(Vec<P>);

impl<T: ?Sized, P: Predicate<T>> Predicate<T> for AnyOf<P> {
    fn check(&self, value: &T) -> bool {
        self.0.iter().any(|p| p.check(value))
    }
}
```

#### String Predicates

```rust
// src/predicate/string.rs

use super::*;
use std::marker::PhantomData;

/// Predicate that checks if a string is not empty.
#[derive(Clone, Copy, Default)]
pub struct NotEmpty;

impl Predicate<str> for NotEmpty {
    #[inline]
    fn check(&self, value: &str) -> bool {
        !value.is_empty()
    }
}

impl Predicate<String> for NotEmpty {
    #[inline]
    fn check(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

pub fn not_empty() -> NotEmpty {
    NotEmpty
}

/// Predicate that checks string length is in range.
#[derive(Clone, Copy)]
pub struct LenBetween {
    min: usize,
    max: usize,
}

impl Predicate<str> for LenBetween {
    #[inline]
    fn check(&self, value: &str) -> bool {
        let len = value.len();
        len >= self.min && len <= self.max
    }
}

pub fn len_between(min: usize, max: usize) -> LenBetween {
    LenBetween { min, max }
}

pub fn len_min(min: usize) -> LenBetween {
    LenBetween { min, max: usize::MAX }
}

pub fn len_max(max: usize) -> LenBetween {
    LenBetween { min: 0, max }
}

pub fn len_eq(len: usize) -> LenBetween {
    LenBetween { min: len, max: len }
}

/// Predicate that checks if string starts with a prefix.
#[derive(Clone)]
pub struct StartsWith<S>(pub S);

impl<S: AsRef<str> + Send + Sync> Predicate<str> for StartsWith<S> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.starts_with(self.0.as_ref())
    }
}

pub fn starts_with<S: AsRef<str> + Send + Sync>(prefix: S) -> StartsWith<S> {
    StartsWith(prefix)
}

/// Predicate that checks if string ends with a suffix.
#[derive(Clone)]
pub struct EndsWith<S>(pub S);

impl<S: AsRef<str> + Send + Sync> Predicate<str> for EndsWith<S> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.ends_with(self.0.as_ref())
    }
}

pub fn ends_with<S: AsRef<str> + Send + Sync>(suffix: S) -> EndsWith<S> {
    EndsWith(suffix)
}

/// Predicate that checks if string contains a substring.
#[derive(Clone)]
pub struct Contains<S>(pub S);

impl<S: AsRef<str> + Send + Sync> Predicate<str> for Contains<S> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.contains(self.0.as_ref())
    }
}

pub fn contains<S: AsRef<str> + Send + Sync>(substring: S) -> Contains<S> {
    Contains(substring)
}

/// Predicate that checks if all characters satisfy a predicate.
#[derive(Clone, Copy)]
pub struct AllChars<F>(pub F);

impl<F: Fn(char) -> bool + Send + Sync> Predicate<str> for AllChars<F> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.chars().all(&self.0)
    }
}

pub fn all_chars<F: Fn(char) -> bool + Send + Sync>(f: F) -> AllChars<F> {
    AllChars(f)
}

/// Predicate that checks if any character satisfies a predicate.
#[derive(Clone, Copy)]
pub struct AnyChar<F>(pub F);

impl<F: Fn(char) -> bool + Send + Sync> Predicate<str> for AnyChar<F> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.chars().any(&self.0)
    }
}

pub fn any_char<F: Fn(char) -> bool + Send + Sync>(f: F) -> AnyChar<F> {
    AnyChar(f)
}

// Convenience predicates
pub fn is_ascii() -> AllChars<fn(char) -> bool> {
    AllChars(char::is_ascii)
}

pub fn is_alphanumeric() -> AllChars<fn(char) -> bool> {
    AllChars(char::is_alphanumeric)
}

pub fn is_alphabetic() -> AllChars<fn(char) -> bool> {
    AllChars(char::is_alphabetic)
}

pub fn is_numeric() -> AllChars<fn(char) -> bool> {
    AllChars(char::is_numeric)
}
```

#### Number Predicates

```rust
// src/predicate/number.rs

use super::*;
use std::cmp::PartialOrd;

/// Predicate for equality.
#[derive(Clone, Copy)]
pub struct Eq<T>(pub T);

impl<T: PartialEq + Send + Sync> Predicate<T> for Eq<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value == self.0
    }
}

pub fn eq<T: PartialEq + Send + Sync>(value: T) -> Eq<T> {
    Eq(value)
}

/// Predicate for greater than.
#[derive(Clone, Copy)]
pub struct Gt<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Gt<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value > self.0
    }
}

pub fn gt<T: PartialOrd + Send + Sync>(value: T) -> Gt<T> {
    Gt(value)
}

/// Predicate for greater than or equal.
#[derive(Clone, Copy)]
pub struct Ge<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Ge<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value >= self.0
    }
}

pub fn ge<T: PartialOrd + Send + Sync>(value: T) -> Ge<T> {
    Ge(value)
}

/// Predicate for less than.
#[derive(Clone, Copy)]
pub struct Lt<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Lt<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value < self.0
    }
}

pub fn lt<T: PartialOrd + Send + Sync>(value: T) -> Lt<T> {
    Lt(value)
}

/// Predicate for less than or equal.
#[derive(Clone, Copy)]
pub struct Le<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Le<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value <= self.0
    }
}

pub fn le<T: PartialOrd + Send + Sync>(value: T) -> Le<T> {
    Le(value)
}

/// Predicate for value in range (inclusive).
#[derive(Clone, Copy)]
pub struct Between<T> {
    min: T,
    max: T,
}

impl<T: PartialOrd + Send + Sync> Predicate<T> for Between<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

pub fn between<T: PartialOrd + Send + Sync>(min: T, max: T) -> Between<T> {
    Between { min, max }
}

/// Positive number predicate.
pub fn positive<T>() -> Gt<T>
where
    T: PartialOrd + Default + Send + Sync,
{
    Gt(T::default())
}

/// Negative number predicate.
pub fn negative<T>() -> Lt<T>
where
    T: PartialOrd + Default + Send + Sync,
{
    Lt(T::default())
}

/// Non-negative number predicate.
pub fn non_negative<T>() -> Ge<T>
where
    T: PartialOrd + Default + Send + Sync,
{
    Ge(T::default())
}
```

#### Validation Integration

```rust
// src/predicate/validation.rs

use super::*;
use crate::Validation;

/// Validate a value using a predicate.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let result = validate("hello", len_min(3), vec!["too short"]);
/// assert_eq!(result, Validation::success("hello"));
/// ```
pub fn validate<T, E, P>(value: T, predicate: P, error: E) -> Validation<T, E>
where
    P: Predicate<T>,
{
    if predicate.check(&value) {
        Validation::success(value)
    } else {
        Validation::failure(error)
    }
}

/// Validate a value with an error factory.
pub fn validate_with<T, E, P, F>(value: T, predicate: P, error_fn: F) -> Validation<T, E>
where
    P: Predicate<T>,
    F: FnOnce(&T) -> E,
{
    if predicate.check(&value) {
        Validation::success(value)
    } else {
        Validation::failure(error_fn(&value))
    }
}

// Extension on Validation
impl<T, E> Validation<T, E> {
    /// Ensure the success value satisfies a predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::{Validation, predicate::*};
    ///
    /// let result = Validation::success("hello")
    ///     .ensure(len_min(3), vec!["too short"]);
    /// ```
    pub fn ensure<P>(self, predicate: P, error: E) -> Validation<T, E>
    where
        P: Predicate<T>,
    {
        match self {
            Validation::Success(value) if predicate.check(&value) => Validation::Success(value),
            Validation::Success(_) => Validation::Failure(error),
            Validation::Failure(e) => Validation::Failure(e),
        }
    }

    /// Ensure with error factory.
    pub fn ensure_with<P, F>(self, predicate: P, error_fn: F) -> Validation<T, E>
    where
        P: Predicate<T>,
        F: FnOnce(&T) -> E,
    {
        match self {
            Validation::Success(ref value) if predicate.check(value) => self,
            Validation::Success(ref value) => Validation::Failure(error_fn(value)),
            Validation::Failure(e) => Validation::Failure(e),
        }
    }
}
```

### Module Structure

```
src/
├── lib.rs                    # Add: pub mod predicate;
├── predicate/
│   ├── mod.rs               # Trait definitions, re-exports
│   ├── combinators.rs       # And, Or, Not, AllOf, AnyOf
│   ├── string.rs            # String predicates
│   ├── number.rs            # Numeric predicates
│   ├── collection.rs        # Collection predicates
│   ├── validation.rs        # Validation integration
│   └── prelude.rs           # Common imports
```

## Dependencies

### Prerequisites
- None

### Affected Components
- `Validation` - add `ensure` method
- Effect (Spec 029) - add `filter_or` method

### External Dependencies
- `regex` crate (optional, behind feature flag)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Logical combinator tests
    #[test]
    fn test_and() {
        let p = gt(0).and(lt(10));
        assert!(p.check(&5));
        assert!(!p.check(&0));
        assert!(!p.check(&10));
    }

    #[test]
    fn test_or() {
        let p = lt(0).or(gt(100));
        assert!(p.check(&-5));
        assert!(p.check(&150));
        assert!(!p.check(&50));
    }

    #[test]
    fn test_not() {
        let p = positive::<i32>().not();
        assert!(p.check(&-5));
        assert!(p.check(&0));
        assert!(!p.check(&5));
    }

    #[test]
    fn test_all_of() {
        let p = all_of([gt(0), lt(100), |x: &i32| x % 2 == 0]);
        assert!(p.check(&50));
        assert!(!p.check(&51)); // odd
        assert!(!p.check(&0));  // not > 0
    }

    // String predicate tests
    #[test]
    fn test_not_empty() {
        assert!(not_empty().check("hello"));
        assert!(!not_empty().check(""));
    }

    #[test]
    fn test_len_between() {
        let p = len_between(3, 10);
        assert!(!p.check("ab"));      // too short
        assert!(p.check("abc"));      // exactly min
        assert!(p.check("hello"));    // in range
        assert!(p.check("1234567890")); // exactly max
        assert!(!p.check("12345678901")); // too long
    }

    #[test]
    fn test_starts_with() {
        assert!(starts_with("http").check("https://example.com"));
        assert!(!starts_with("http").check("ftp://example.com"));
    }

    #[test]
    fn test_all_chars() {
        assert!(all_chars(char::is_alphabetic).check("hello"));
        assert!(!all_chars(char::is_alphabetic).check("hello123"));
    }

    // Number predicate tests
    #[test]
    fn test_between() {
        let p = between(0, 100);
        assert!(p.check(&0));
        assert!(p.check(&50));
        assert!(p.check(&100));
        assert!(!p.check(&-1));
        assert!(!p.check(&101));
    }

    #[test]
    fn test_positive() {
        let p = positive::<i32>();
        assert!(p.check(&1));
        assert!(!p.check(&0));
        assert!(!p.check(&-1));
    }

    // Chaining tests
    #[test]
    fn test_complex_chain() {
        let valid_username = not_empty()
            .and(len_between(3, 20))
            .and(all_chars(|c: char| c.is_alphanumeric() || c == '_'));

        assert!(valid_username.check("john_doe"));
        assert!(valid_username.check("a_1"));
        assert!(!valid_username.check("ab"));  // too short
        assert!(!valid_username.check("invalid-name")); // contains hyphen
    }

    // Validation integration tests
    #[test]
    fn test_validate_success() {
        let result = validate("hello", len_min(3), "too short");
        assert_eq!(result, Validation::success("hello"));
    }

    #[test]
    fn test_validate_failure() {
        let result = validate("hi", len_min(3), "too short");
        assert_eq!(result, Validation::failure("too short"));
    }

    #[test]
    fn test_ensure() {
        let result = Validation::success("hello")
            .ensure(len_min(3), "too short")
            .ensure(len_max(10), "too long");
        assert_eq!(result, Validation::success("hello"));

        let result = Validation::success("hi")
            .ensure(len_min(3), "too short");
        assert_eq!(result, Validation::failure("too short"));
    }

    // Closure predicate tests
    #[test]
    fn test_closure_as_predicate() {
        let is_even = |x: &i32| x % 2 == 0;
        assert!(is_even.check(&4));
        assert!(!is_even.check(&3));

        // Can be combined
        let even_and_positive = is_even.and(positive());
        assert!(even_and_positive.check(&4));
        assert!(!even_and_positive.check(&-4));
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
        // De Morgan's laws
        #[test]
        fn prop_demorgan_and(x: i32) {
            let p1 = gt(0);
            let p2 = lt(100);

            // not(p1 and p2) == (not p1) or (not p2)
            let left = p1.and(p2).not();
            let right = p1.not().or(p2.not());

            prop_assert_eq!(left.check(&x), right.check(&x));
        }

        #[test]
        fn prop_demorgan_or(x: i32) {
            let p1 = gt(0);
            let p2 = lt(100);

            // not(p1 or p2) == (not p1) and (not p2)
            let left = p1.or(p2).not();
            let right = p1.not().and(p2.not());

            prop_assert_eq!(left.check(&x), right.check(&x));
        }

        // Double negation
        #[test]
        fn prop_double_negation(x: i32) {
            let p = gt(0);
            prop_assert_eq!(p.not().not().check(&x), p.check(&x));
        }

        // Idempotence
        #[test]
        fn prop_and_idempotent(x: i32) {
            let p = gt(0);
            prop_assert_eq!(p.and(gt(0)).check(&x), p.check(&x));
        }

        #[test]
        fn prop_or_idempotent(x: i32) {
            let p = gt(0);
            prop_assert_eq!(p.or(gt(0)).check(&x), p.check(&x));
        }
    }
}
```

## Documentation Requirements

### Code Documentation

- Full rustdoc on `Predicate` trait
- Examples for each predicate function
- Module-level documentation with overview
- "When to use" guidance

### User Documentation

Add to guide:

```markdown
## Predicate Combinators

Build complex validation logic from simple, reusable predicates:

```rust
use stillwater::predicate::*;

// Define reusable predicates
let valid_email_length = len_between(5, 254);
let has_at_sign = contains("@");
let no_spaces = all_chars(|c| c != ' ');

// Combine them
let basic_email = valid_email_length
    .and(has_at_sign)
    .and(no_spaces);

// Use in validation
let result = validate(email, basic_email, "Invalid email format");
```
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Trait-based | Allows both structs and closures |
| Send + Sync required | Thread-safe by default |
| Blanket impl for Fn | Closures work naturally |
| Copy where possible | Predicates are cheap to copy |

### Future Enhancements

1. **Regex support**: `matches(pattern)` behind feature flag
2. **Error messages**: Predicates that carry their error message
3. **Async predicates**: For validation that requires I/O
4. **Derive macro**: `#[derive(Predicate)]` for custom types

## Migration and Compatibility

- **Breaking changes**: None (new module)
- **Additions to existing types**: `Validation::ensure` method

---

*"Build complex validation from simple, composable pieces."*
