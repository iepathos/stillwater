---
number: 026
title: Either<L, R> Sum Type
category: foundation
priority: high
status: ready
dependencies: []
created: 2025-11-27
---

# Specification 026: Either<L, R> Sum Type

**Category**: foundation
**Priority**: high
**Status**: ready
**Dependencies**: None

## Context

### The Gap

Rust's `Result<T, E>` type is semantically tied to success/failure. The type names (`Ok`/`Err`) and ecosystem conventions make it awkward to use `Result` for genuine branching where neither side represents an error.

Consider these use cases that don't fit `Result` semantics:

```rust
// Awkward: neither Cache nor Database is an "error"
fn get_user(id: UserId) -> Result<CachedUser, DatabaseUser>  // Misleading!

// What we actually want:
fn get_user(id: UserId) -> Either<CachedUser, DatabaseUser>  // Clear intent
```

### Common FP Pattern

`Either<L, R>` is a fundamental type in functional programming:

- **Haskell**: `Either a b = Left a | Right b`
- **Scala**: `Either[A, B]`
- **TypeScript (fp-ts)**: `Either<E, A>`
- **Rust crates**: `either` crate has 50M+ downloads

The type represents a value that is one of two possible types, without the success/failure connotation.

### Use Cases

1. **Branching without error semantics**: Caching (cached vs fresh), routing (local vs remote)
2. **Sum type encoding**: Representing "one of two things" in APIs
3. **Validation results**: Sometimes you want to preserve both valid and invalid items
4. **Parser combinators**: Different parse paths
5. **Configuration**: Default vs custom values

## Objective

Add an `Either<L, R>` type to Stillwater that provides a semantically neutral sum type with comprehensive functional programming combinators, integrating well with the existing `Validation` and `Effect` types.

## Requirements

### Functional Requirements

#### FR1: Core Type Definition

- **MUST** define `Either<L, R>` enum with `Left(L)` and `Right(R)` variants
- **MUST** implement `Clone`, `Copy` (when inner types allow), `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`
- **MUST** be `Send` and `Sync` when inner types are

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
```

#### FR2: Constructors and Predicates

- **MUST** provide `Either::left(value)` constructor
- **MUST** provide `Either::right(value)` constructor
- **MUST** provide `is_left(&self) -> bool`
- **MUST** provide `is_right(&self) -> bool`
- **MUST** provide `into_left(self) -> Option<L>`
- **MUST** provide `into_right(self) -> Option<R>`
- **MUST** provide `as_ref(&self) -> Either<&L, &R>`
- **MUST** provide `as_mut(&mut self) -> Either<&mut L, &mut R>`

#### FR3: Transformation Methods

- **MUST** provide `map_left<F>(self, f: F) -> Either<L2, R>`
- **MUST** provide `map_right<F>(self, f: F) -> Either<L, R2>`
- **MUST** provide `map<F>(self, f: F) -> Either<L, R2>` (alias for map_right, right-biased)
- **MUST** provide `bimap<F, G>(self, f: F, g: G) -> Either<L2, R2>`
- **MUST** provide `swap(self) -> Either<R, L>`

#### FR4: Folding and Extraction

- **MUST** provide `fold<T, F, G>(self, left_fn: F, right_fn: G) -> T`
- **MUST** provide `unwrap_left(self) -> L` (panics if Right)
- **MUST** provide `unwrap_right(self) -> R` (panics if Left)
- **MUST** provide `expect_left(self, msg: &str) -> L`
- **MUST** provide `expect_right(self, msg: &str) -> R`
- **MUST** provide `left_or(self, default: L) -> L`
- **MUST** provide `right_or(self, default: R) -> R`
- **MUST** provide `left_or_else<F>(self, f: F) -> L`
- **MUST** provide `right_or_else<F>(self, f: F) -> R`

#### FR5: Monadic Operations (Right-Biased)

- **MUST** provide `and_then<F>(self, f: F) -> Either<L, R2>` (right-biased)
- **MUST** provide `or_else<F>(self, f: F) -> Either<L2, R>` (for Left values)
- **MUST** provide `flatten(self) -> Either<L, R>` where R = Either<L, R>

#### FR6: Conversions

- **MUST** provide `From<Result<R, L>>` for `Either<L, R>`
- **MUST** provide `Into<Result<R, L>>` for `Either<L, R>`
- **MUST** provide `into_result(self) -> Result<R, L>` (explicit conversion)
- **MUST** provide `from_result(result: Result<R, L>) -> Self`
- **SHOULD** provide `into_validation(self) -> Validation<R, L>` for integration

#### FR7: Iterator Methods

- **MUST** provide `iter(&self) -> impl Iterator<Item = &R>` (right values only)
- **MUST** provide `iter_mut(&mut self) -> impl Iterator<Item = &mut R>`
- **MUST** provide `into_iter(self) -> impl Iterator<Item = R>`
- **MUST** implement `IntoIterator` for `Either<L, R>`

#### FR8: Collection Utilities

- **MUST** provide `partition<I>(iter: I) -> (Vec<L>, Vec<R>)` where I: Iterator<Item = Either<L, R>>
- **MUST** provide `lefts<I>(iter: I) -> impl Iterator<Item = L>`
- **MUST** provide `rights<I>(iter: I) -> impl Iterator<Item = R>`

### Non-Functional Requirements

#### NFR1: Zero-Cost

- All methods MUST be zero-cost abstractions (no heap allocation)
- `#[inline]` hints SHOULD be used for small methods
- Size of `Either<L, R>` MUST equal `max(size_of::<L>(), size_of::<R>()) + discriminant`

#### NFR2: Ergonomics

- Type inference SHOULD work naturally in common patterns
- Method chaining SHOULD feel natural
- Pattern matching SHOULD work seamlessly

#### NFR3: Serde Support (Optional)

- SHOULD support `#[derive(Serialize, Deserialize)]` behind `serde` feature
- Serialization format SHOULD be `{"Left": value}` or `{"Right": value}`

## Acceptance Criteria

### Core Type

- [ ] **AC1**: `Either<L, R>` enum compiles with all standard derives
- [ ] **AC2**: `Either::left(42)` creates `Either::Left(42)`
- [ ] **AC3**: `Either::right("hello")` creates `Either::Right("hello")`
- [ ] **AC4**: `is_left()` and `is_right()` return correct booleans
- [ ] **AC5**: `into_left()` and `into_right()` return correct Options

### Transformations

- [ ] **AC6**: `map_left` transforms Left values, passes Right through
- [ ] **AC7**: `map_right` transforms Right values, passes Left through
- [ ] **AC8**: `bimap` transforms both variants appropriately
- [ ] **AC9**: `swap` exchanges Left and Right

### Folding

- [ ] **AC10**: `fold` works with both variants
- [ ] **AC11**: `unwrap_left` returns value or panics appropriately
- [ ] **AC12**: `unwrap_right` returns value or panics appropriately
- [ ] **AC13**: `left_or` and `right_or` provide defaults

### Monadic Operations

- [ ] **AC14**: `and_then` chains operations on Right values
- [ ] **AC15**: `and_then` passes Left values through unchanged
- [ ] **AC16**: `or_else` chains operations on Left values
- [ ] **AC17**: `flatten` reduces nested Either

### Conversions

- [ ] **AC18**: `Result` converts to `Either` and back losslessly
- [ ] **AC19**: `into_validation` integration works

### Collection Utilities

- [ ] **AC20**: `partition` correctly separates Lefts and Rights
- [ ] **AC21**: `lefts` and `rights` filter correctly

### Property Tests

- [ ] **AC22**: Functor law: `e.map(id) == e`
- [ ] **AC23**: Functor law: `e.map(f).map(g) == e.map(|x| g(f(x)))`
- [ ] **AC24**: `swap.swap` is identity
- [ ] **AC25**: `bimap(f, g).swap == swap.bimap(g, f)`

## Technical Details

### Implementation

```rust
// src/either.rs

/// A value that is either `Left(L)` or `Right(R)`.
///
/// `Either` is a general-purpose sum type with no inherent success/failure semantics.
/// Unlike `Result`, neither variant implies an error condition.
///
/// By convention, `Either` is "right-biased": methods like `map` and `and_then`
/// operate on the `Right` variant. This matches the common FP convention where
/// `Right` is the "happy path" in computations.
///
/// # When to Use
///
/// Use `Either` instead of `Result` when:
/// - Neither variant represents an error (e.g., cached vs fresh data)
/// - You need a sum type without error semantics
/// - You want to preserve both "sides" of a computation
///
/// Use `Result` when:
/// - One variant clearly represents failure/error
/// - You want to use `?` operator for early returns
///
/// # Example
///
/// ```rust
/// use stillwater::Either;
///
/// // Representing two valid data sources
/// enum DataSource {
///     Cache,
///     Database,
/// }
///
/// fn get_user(from_cache: bool) -> Either<CachedUser, FreshUser> {
///     if from_cache {
///         Either::left(CachedUser { /* ... */ })
///     } else {
///         Either::right(FreshUser { /* ... */ })
///     }
/// }
///
/// // Process based on source
/// let user = get_user(true);
/// let name = user.fold(
///     |cached| cached.name.clone(),
///     |fresh| fresh.fetch_name(),
/// );
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Either<L, R> {
    /// The left variant
    Left(L),
    /// The right variant
    Right(R),
}

impl<L, R> Either<L, R> {
    // ========== Constructors ==========

    /// Create a Left value.
    #[inline]
    pub fn left(value: L) -> Self {
        Either::Left(value)
    }

    /// Create a Right value.
    #[inline]
    pub fn right(value: R) -> Self {
        Either::Right(value)
    }

    // ========== Predicates ==========

    /// Returns `true` if this is a `Left` value.
    #[inline]
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Returns `true` if this is a `Right` value.
    #[inline]
    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }

    // ========== Extractors ==========

    /// Returns the left value if present, consuming self.
    #[inline]
    pub fn into_left(self) -> Option<L> {
        match self {
            Either::Left(l) => Some(l),
            Either::Right(_) => None,
        }
    }

    /// Returns the right value if present, consuming self.
    #[inline]
    pub fn into_right(self) -> Option<R> {
        match self {
            Either::Left(_) => None,
            Either::Right(r) => Some(r),
        }
    }

    /// Convert to `Either<&L, &R>`.
    #[inline]
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    /// Convert to `Either<&mut L, &mut R>`.
    #[inline]
    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    // ========== Transformations ==========

    /// Transform the left value.
    #[inline]
    pub fn map_left<L2, F>(self, f: F) -> Either<L2, R>
    where
        F: FnOnce(L) -> L2,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(r),
        }
    }

    /// Transform the right value.
    #[inline]
    pub fn map_right<R2, F>(self, f: F) -> Either<L, R2>
    where
        F: FnOnce(R) -> R2,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => Either::Right(f(r)),
        }
    }

    /// Transform the right value (right-biased `map`).
    ///
    /// This is an alias for `map_right` following the right-biased convention.
    #[inline]
    pub fn map<R2, F>(self, f: F) -> Either<L, R2>
    where
        F: FnOnce(R) -> R2,
    {
        self.map_right(f)
    }

    /// Transform both variants.
    #[inline]
    pub fn bimap<L2, R2, F, G>(self, f: F, g: G) -> Either<L2, R2>
    where
        F: FnOnce(L) -> L2,
        G: FnOnce(R) -> R2,
    {
        match self {
            Either::Left(l) => Either::Left(f(l)),
            Either::Right(r) => Either::Right(g(r)),
        }
    }

    /// Swap Left and Right.
    #[inline]
    pub fn swap(self) -> Either<R, L> {
        match self {
            Either::Left(l) => Either::Right(l),
            Either::Right(r) => Either::Left(r),
        }
    }

    // ========== Folding ==========

    /// Fold both variants into a single value.
    #[inline]
    pub fn fold<T, F, G>(self, left_fn: F, right_fn: G) -> T
    where
        F: FnOnce(L) -> T,
        G: FnOnce(R) -> T,
    {
        match self {
            Either::Left(l) => left_fn(l),
            Either::Right(r) => right_fn(r),
        }
    }

    /// Extract the left value, panicking if Right.
    #[inline]
    pub fn unwrap_left(self) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => panic!("called `Either::unwrap_left()` on a `Right` value"),
        }
    }

    /// Extract the right value, panicking if Left.
    #[inline]
    pub fn unwrap_right(self) -> R {
        match self {
            Either::Left(_) => panic!("called `Either::unwrap_right()` on a `Left` value"),
            Either::Right(r) => r,
        }
    }

    /// Extract the left value with a custom panic message.
    #[inline]
    pub fn expect_left(self, msg: &str) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => panic!("{}", msg),
        }
    }

    /// Extract the right value with a custom panic message.
    #[inline]
    pub fn expect_right(self, msg: &str) -> R {
        match self {
            Either::Left(_) => panic!("{}", msg),
            Either::Right(r) => r,
        }
    }

    /// Return the left value or a default.
    #[inline]
    pub fn left_or(self, default: L) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => default,
        }
    }

    /// Return the right value or a default.
    #[inline]
    pub fn right_or(self, default: R) -> R {
        match self {
            Either::Left(_) => default,
            Either::Right(r) => r,
        }
    }

    /// Return the left value or compute it from the right.
    #[inline]
    pub fn left_or_else<F>(self, f: F) -> L
    where
        F: FnOnce(R) -> L,
    {
        match self {
            Either::Left(l) => l,
            Either::Right(r) => f(r),
        }
    }

    /// Return the right value or compute it from the left.
    #[inline]
    pub fn right_or_else<F>(self, f: F) -> R
    where
        F: FnOnce(L) -> R,
    {
        match self {
            Either::Left(l) => f(l),
            Either::Right(r) => r,
        }
    }

    // ========== Monadic Operations (Right-Biased) ==========

    /// Chain a computation on the right value.
    #[inline]
    pub fn and_then<R2, F>(self, f: F) -> Either<L, R2>
    where
        F: FnOnce(R) -> Either<L, R2>,
    {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(r) => f(r),
        }
    }

    /// Chain a computation on the left value.
    #[inline]
    pub fn or_else<L2, F>(self, f: F) -> Either<L2, R>
    where
        F: FnOnce(L) -> Either<L2, R>,
    {
        match self {
            Either::Left(l) => f(l),
            Either::Right(r) => Either::Right(r),
        }
    }

    // ========== Conversions ==========

    /// Convert to Result (Right becomes Ok, Left becomes Err).
    #[inline]
    pub fn into_result(self) -> Result<R, L> {
        match self {
            Either::Left(l) => Err(l),
            Either::Right(r) => Ok(r),
        }
    }

    /// Create from Result (Ok becomes Right, Err becomes Left).
    #[inline]
    pub fn from_result(result: Result<R, L>) -> Self {
        match result {
            Ok(r) => Either::Right(r),
            Err(l) => Either::Left(l),
        }
    }
}

// Flatten for nested Either
impl<L, R> Either<L, Either<L, R>> {
    /// Flatten a nested Either.
    #[inline]
    pub fn flatten(self) -> Either<L, R> {
        match self {
            Either::Left(l) => Either::Left(l),
            Either::Right(inner) => inner,
        }
    }
}

// ========== Trait Implementations ==========

// Note: Debug is derived via #[derive(Debug)] on the enum definition.
// The derived impl produces equivalent output to a manual implementation.

impl<L, R> From<Result<R, L>> for Either<L, R> {
    fn from(result: Result<R, L>) -> Self {
        Either::from_result(result)
    }
}

impl<L, R> From<Either<L, R>> for Result<R, L> {
    fn from(either: Either<L, R>) -> Self {
        either.into_result()
    }
}

impl<L, R> Default for Either<L, R>
where
    R: Default,
{
    /// Returns `Either::Right(R::default())`.
    fn default() -> Self {
        Either::Right(R::default())
    }
}

// ========== Iterator Support ==========

impl<L, R> Either<L, R> {
    /// Returns an iterator over the right value, if present.
    ///
    /// This is right-biased: only `Right` values yield an element.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &R> {
        self.as_ref().into_right().into_iter()
    }

    /// Returns a mutable iterator over the right value, if present.
    ///
    /// This is right-biased: only `Right` values yield an element.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut R> {
        self.as_mut().into_right().into_iter()
    }
}

impl<L, R> IntoIterator for Either<L, R> {
    type Item = R;
    type IntoIter = std::option::IntoIter<R>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_right().into_iter()
    }
}

impl<'a, L, R> IntoIterator for &'a Either<L, R> {
    type Item = &'a R;
    type IntoIter = std::option::IntoIter<&'a R>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().into_right().into_iter()
    }
}

// ========== Collection Utilities ==========

/// Partition an iterator of Either into two vectors.
pub fn partition<L, R, I>(iter: I) -> (Vec<L>, Vec<R>)
where
    I: IntoIterator<Item = Either<L, R>>,
{
    let mut lefts = Vec::new();
    let mut rights = Vec::new();

    for item in iter {
        match item {
            Either::Left(l) => lefts.push(l),
            Either::Right(r) => rights.push(r),
        }
    }

    (lefts, rights)
}

/// Extract all Left values from an iterator.
pub fn lefts<L, R, I>(iter: I) -> impl Iterator<Item = L>
where
    I: IntoIterator<Item = Either<L, R>>,
{
    iter.into_iter().filter_map(|e| e.into_left())
}

/// Extract all Right values from an iterator.
pub fn rights<L, R, I>(iter: I) -> impl Iterator<Item = R>
where
    I: IntoIterator<Item = Either<L, R>>,
{
    iter.into_iter().filter_map(|e| e.into_right())
}
```

### Module Structure

```
src/
├── lib.rs          # Add: pub mod either;
├── either.rs       # New file
```

### Integration with Validation

```rust
// In either.rs

impl<L, R> Either<L, R> {
    /// Convert to Validation (Right becomes Success, Left becomes Failure).
    #[inline]
    pub fn into_validation(self) -> crate::Validation<R, L> {
        match self {
            Either::Left(l) => crate::Validation::Failure(l),
            Either::Right(r) => crate::Validation::Success(r),
        }
    }
}

impl<T, E> crate::Validation<T, E> {
    /// Convert to Either (Success becomes Right, Failure becomes Left).
    #[inline]
    pub fn into_either(self) -> Either<E, T> {
        match self {
            crate::Validation::Success(t) => Either::Right(t),
            crate::Validation::Failure(e) => Either::Left(e),
        }
    }
}
```

## Dependencies

### Prerequisites
- None

### Affected Components
- `lib.rs` - Add module export
- `Validation` - Add `into_either()` method

### External Dependencies
- None (optional serde behind feature flag)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        assert!(Either::<i32, &str>::left(42).is_left());
        assert!(Either::<i32, &str>::right("hello").is_right());
    }

    #[test]
    fn test_map_left() {
        let e: Either<i32, &str> = Either::left(21);
        assert_eq!(e.map_left(|x| x * 2), Either::left(42));

        let e: Either<i32, &str> = Either::right("hello");
        assert_eq!(e.map_left(|x| x * 2), Either::right("hello"));
    }

    #[test]
    fn test_map_right() {
        let e: Either<i32, i32> = Either::right(21);
        assert_eq!(e.map_right(|x| x * 2), Either::right(42));

        let e: Either<i32, i32> = Either::left(100);
        assert_eq!(e.map_right(|x| x * 2), Either::left(100));
    }

    #[test]
    fn test_bimap() {
        let e: Either<i32, &str> = Either::left(1);
        assert_eq!(e.bimap(|x| x + 1, |s| s.len()), Either::left(2));

        let e: Either<i32, &str> = Either::right("hello");
        assert_eq!(e.bimap(|x| x + 1, |s| s.len()), Either::right(5));
    }

    #[test]
    fn test_swap() {
        let e: Either<i32, &str> = Either::left(42);
        assert_eq!(e.swap(), Either::right(42));

        let e: Either<i32, &str> = Either::right("hello");
        assert_eq!(e.swap(), Either::left("hello"));
    }

    #[test]
    fn test_fold() {
        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.fold(|x| x.to_string(), |s| s.to_string()), "42");

        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.fold(|x| x.to_string(), |s| s.to_string()), "hello");
    }

    #[test]
    fn test_and_then() {
        let e: Either<&str, i32> = Either::right(21);
        assert_eq!(e.and_then(|x| Either::right(x * 2)), Either::right(42));

        let e: Either<&str, i32> = Either::left("error");
        assert_eq!(e.and_then(|x| Either::right(x * 2)), Either::left("error"));
    }

    #[test]
    fn test_or_else() {
        let e: Either<i32, &str> = Either::left(1);
        assert_eq!(e.or_else(|_| Either::right("recovered")), Either::right("recovered"));

        let e: Either<i32, &str> = Either::right("ok");
        assert_eq!(e.or_else(|x| Either::left(x * 2)), Either::right("ok"));
    }

    #[test]
    fn test_result_conversion() {
        let ok: Result<i32, &str> = Ok(42);
        let either: Either<&str, i32> = ok.into();
        assert_eq!(either, Either::right(42));

        let err: Result<i32, &str> = Err("error");
        let either: Either<&str, i32> = err.into();
        assert_eq!(either, Either::left("error"));

        // Round-trip
        let original: Either<&str, i32> = Either::right(42);
        let result: Result<i32, &str> = original.into();
        let back: Either<&str, i32> = result.into();
        assert_eq!(back, Either::right(42));
    }

    #[test]
    fn test_partition() {
        let items = vec![
            Either::left(1),
            Either::right("a"),
            Either::left(2),
            Either::right("b"),
        ];

        let (lefts, rights) = partition(items);
        assert_eq!(lefts, vec![1, 2]);
        assert_eq!(rights, vec!["a", "b"]);
    }

    #[test]
    fn test_flatten() {
        let nested: Either<&str, Either<&str, i32>> = Either::right(Either::right(42));
        assert_eq!(nested.flatten(), Either::right(42));

        let nested: Either<&str, Either<&str, i32>> = Either::right(Either::left("inner"));
        assert_eq!(nested.flatten(), Either::left("inner"));

        let nested: Either<&str, Either<&str, i32>> = Either::left("outer");
        assert_eq!(nested.flatten(), Either::left("outer"));
    }

    #[test]
    fn test_into_left_into_right() {
        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.into_left(), Some(42));

        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.into_right(), Some("hello"));

        let left: Either<i32, &str> = Either::left(42);
        assert_eq!(left.into_right(), None);

        let right: Either<i32, &str> = Either::right("hello");
        assert_eq!(right.into_left(), None);
    }

    #[test]
    fn test_iter() {
        let right: Either<&str, i32> = Either::right(42);
        let collected: Vec<_> = right.iter().collect();
        assert_eq!(collected, vec![&42]);

        let left: Either<&str, i32> = Either::left("error");
        let collected: Vec<_> = left.iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_iter_mut() {
        let mut right: Either<&str, i32> = Either::right(42);
        for val in right.iter_mut() {
            *val *= 2;
        }
        assert_eq!(right, Either::right(84));

        let mut left: Either<&str, i32> = Either::left("error");
        for val in left.iter_mut() {
            *val *= 2; // This won't run
        }
        assert_eq!(left, Either::left("error"));
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
        fn prop_swap_involution(x: i32) {
            let e: Either<i32, i32> = Either::left(x);
            prop_assert_eq!(e.swap().swap(), e);

            let e: Either<i32, i32> = Either::right(x);
            prop_assert_eq!(e.swap().swap(), e);
        }

        #[test]
        fn prop_functor_identity(x: i32) {
            let e: Either<(), i32> = Either::right(x);
            prop_assert_eq!(e.map(|v| v), Either::right(x));
        }

        #[test]
        fn prop_functor_composition(x: i32) {
            let f = |v: i32| v + 1;
            let g = |v: i32| v * 2;

            let e: Either<(), i32> = Either::right(x);
            prop_assert_eq!(
                e.map(f).map(g),
                Either::right(x).map(|v| g(f(v)))
            );
        }

        #[test]
        fn prop_result_roundtrip(x: i32) {
            let either: Either<(), i32> = Either::right(x);
            let result: Result<i32, ()> = either.into();
            let back: Either<(), i32> = result.into();
            prop_assert_eq!(back, Either::right(x));
        }
    }
}
```

## Documentation Requirements

### Code Documentation
- Full rustdoc on `Either` type with usage examples
- Rustdoc on all methods with examples
- Module-level documentation explaining when to use `Either` vs `Result`

### User Documentation
- Add to PHILOSOPHY.md: section on `Either` as neutral sum type
- Add example in examples/ directory
- Update README if needed

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Right-biased | Follows Scala/fp-ts convention; `Right` is the "happy path" |
| No `?` operator | Would conflict with Result's error semantics |
| Derives over manual impls | Simpler, correct by construction |
| `into_left()` returns `Option` | Consistent with `Result::ok()` / `Result::err()`, avoids name collision with constructor |

### Naming Convention

The constructors `Either::left(value)` and `Either::right(value)` are associated functions, while the extractors `into_left()` and `into_right()` are methods that consume self. This avoids the naming conflict present in the `either` crate where both constructor and extractor share the same name.

## Migration and Compatibility

- **Breaking changes**: None (new type)
- **External crates**: Consider providing `impl From<either::Either>` behind a feature flag for interop with the popular `either` crate

---

*"When you need two possibilities without the baggage of success and failure."*
