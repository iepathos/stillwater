---
number: 041
title: Validation bimap and bifold Combinators
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-11-27
---

# Specification 041: Validation bimap and bifold Combinators

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None

## Context

### The Problem

Stillwater's `Validation<T, E>` type currently has `map` (for success) and `map_err` (for error), but lacks combinators for working with both sides simultaneously:

```rust
// Current: Verbose two-step transformation
let result = validation
    .map(transform_success)
    .map_err(transform_error);

// What we want: Single transformation of both
let result = validation.bimap(transform_error, transform_success);

// Current: Verbose pattern match to extract single value
let value = match validation {
    Validation::Success(s) => process_success(s),
    Validation::Failure(e) => process_failure(e),
};

// What we want: Direct folding
let value = validation.fold(process_failure, process_success);
```

### Standard FP Operations

`bimap` and `fold` are fundamental operations on sum types:

- **bimap**: Transform both variants with respective functions (Bifunctor)
- **fold**: Collapse to a single value by handling both cases (Catamorphism)

These are standard in:
- **Haskell**: `bimap`, `either` (fold)
- **Scala**: `bimap`, `fold`
- **TypeScript (fp-ts)**: `bimap`, `fold`
- **Rust `either` crate**: `either`, `either_into` (fold equivalents)

### Benefits

1. **Conciseness**: Transform both sides in one operation
2. **Readability**: Intent is clearer than chained map/map_err
3. **Consistency**: Matches `Either` which already has `bimap` (Spec 026)
4. **Completeness**: Makes `Validation` a proper Bifunctor

## Objective

Add `bimap`, `bimap_err`, `fold`, and related combinators to `Validation<T, E>` for comprehensive sum type manipulation, completing the Bifunctor interface.

## Requirements

### Functional Requirements

#### FR1: Validation.bimap Combinator

- **MUST** provide `bimap(f_err, f_success)` method
- **MUST** apply `f_err` to Failure, `f_success` to Success
- **MUST** return `Validation<U, E2>` with transformed types
- **SHOULD** follow Bifunctor convention (left function first)

```rust
fn bimap<U, E2, F, G>(self, f: F, g: G) -> Validation<U, E2>
where
    F: FnOnce(E) -> E2,
    G: FnOnce(T) -> U;
```

#### FR2: Validation.bimap_success_first Alias

- **SHOULD** provide `bimap_success_first(f_success, f_err)` for alternative ordering
- **MAY** be named `transform` for clarity

```rust
fn bimap_success_first<U, E2, F, G>(self, f: F, g: G) -> Validation<U, E2>
where
    F: FnOnce(T) -> U,
    G: FnOnce(E) -> E2;
```

#### FR3: Validation.fold Combinator

- **MUST** provide `fold(on_failure, on_success)` method
- **MUST** collapse Validation to single value of type `R`
- **MUST** call exactly one of the two functions
- **SHOULD** follow standard convention (failure/left handler first)

```rust
fn fold<R, F, G>(self, on_failure: F, on_success: G) -> R
where
    F: FnOnce(E) -> R,
    G: FnOnce(T) -> R;
```

#### FR4: Validation.fold_success_first Alias

- **SHOULD** provide `fold_success_first(on_success, on_failure)` for alternative ordering
- **MAY** be named `match_with` for clarity

```rust
fn fold_success_first<R, F, G>(self, on_success: F, on_failure: G) -> R
where
    F: FnOnce(T) -> R,
    G: FnOnce(E) -> R;
```

#### FR5: Validation.bifold Combinator

- **MAY** provide `bifold(seed, f_err, f_success)` for accumulating fold
- Useful when combining with a starting value

```rust
fn bifold<R, F, G>(self, seed: R, f: F, g: G) -> R
where
    F: FnOnce(R, E) -> R,
    G: FnOnce(R, T) -> R;
```

#### FR6: Validation.unwrap_or_else Enhancement

- **MUST** ensure `unwrap_or_else` exists and is consistent
- **SHOULD** accept function that transforms error to success type

```rust
fn unwrap_or_else<F>(self, f: F) -> T
where
    F: FnOnce(E) -> T;
```

#### FR7: Validation.merge for Same Types

- **MUST** provide `merge()` when `T == E`
- Returns the inner value regardless of variant

```rust
impl<T> Validation<T, T> {
    fn merge(self) -> T;
}
```

### Non-Functional Requirements

#### NFR1: Zero-Cost

- All methods MUST be zero-cost (inline-able, no allocation)
- `#[inline]` hints SHOULD be used

#### NFR2: Consistency

- Naming SHOULD match `Either` type (Spec 026)
- Parameter order SHOULD match FP conventions

#### NFR3: Documentation

- Each method MUST have comprehensive rustdoc
- Examples MUST demonstrate common use cases

## Acceptance Criteria

### bimap

- [ ] **AC1**: `bimap` method exists on `Validation<T, E>`
- [ ] **AC2**: `Success(5).bimap(|e| e, |x| x * 2)` returns `Success(10)`
- [ ] **AC3**: `Failure("err").bimap(|e| e.len(), |x| x)` returns `Failure(3)`
- [ ] **AC4**: Types transform correctly: `Validation<i32, String>` -> `Validation<f64, usize>`

### fold

- [ ] **AC5**: `fold` method exists on `Validation<T, E>`
- [ ] **AC6**: `Success(5).fold(|_| 0, |x| x * 2)` returns `10`
- [ ] **AC7**: `Failure("err").fold(|e| e.len() as i32, |_| 0)` returns `3`
- [ ] **AC8**: Return type is unified from both branches

### unwrap_or_else

- [ ] **AC9**: `unwrap_or_else` transforms failure to success type
- [ ] **AC10**: `Failure("err").unwrap_or_else(|e| e.len())` returns `3`
- [ ] **AC11**: `Success(5).unwrap_or_else(|_| 0)` returns `5`

### merge

- [ ] **AC12**: `merge` method exists when `T == E`
- [ ] **AC13**: `Success::<i32, i32>(5).merge()` returns `5`
- [ ] **AC14**: `Failure::<i32, i32>(3).merge()` returns `3`

### Integration

- [ ] **AC15**: Works in chained operations with `map`, `and_then`
- [ ] **AC16**: Works with Semigroup error types
- [ ] **AC17**: Consistent with `Either::bimap` and `Either::fold`

## Technical Details

### Implementation

```rust
// In src/validation/core.rs

impl<T, E> Validation<T, E> {
    // ========== bimap ==========

    /// Transform both variants of this Validation.
    ///
    /// Applies `f` to the error if Failure, or `g` to the value if Success.
    /// This is the Bifunctor `bimap` operation.
    ///
    /// # Arguments
    ///
    /// * `f` - Function to transform the error (Failure case)
    /// * `g` - Function to transform the value (Success case)
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let success = Validation::<_, String>::success(5);
    /// let result = success.bimap(|e| e.len(), |x| x * 2);
    /// assert_eq!(result, Validation::Success(10));
    ///
    /// let failure = Validation::<i32, _>::failure("error".to_string());
    /// let result = failure.bimap(|e| e.len(), |x| x * 2);
    /// assert_eq!(result, Validation::Failure(5));
    /// ```
    #[inline]
    pub fn bimap<U, E2, F, G>(self, f: F, g: G) -> Validation<U, E2>
    where
        F: FnOnce(E) -> E2,
        G: FnOnce(T) -> U,
    {
        match self {
            Validation::Success(value) => Validation::Success(g(value)),
            Validation::Failure(error) => Validation::Failure(f(error)),
        }
    }

    /// Transform both variants with success function first.
    ///
    /// This is an alternative to `bimap` with a more intuitive argument order
    /// for those who think "success first".
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(5);
    /// let result = v.bimap_success_first(|x| x * 2, |e| e.len());
    /// assert_eq!(result, Validation::Success(10));
    /// ```
    #[inline]
    pub fn bimap_success_first<U, E2, F, G>(self, f: F, g: G) -> Validation<U, E2>
    where
        F: FnOnce(T) -> U,
        G: FnOnce(E) -> E2,
    {
        self.bimap(g, f)
    }

    // ========== fold ==========

    /// Fold this Validation into a single value.
    ///
    /// Collapses the Validation by applying the appropriate function based
    /// on the variant. This is the catamorphism for Validation.
    ///
    /// # Arguments
    ///
    /// * `on_failure` - Function to handle Failure case
    /// * `on_success` - Function to handle Success case
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let success = Validation::<_, String>::success(5);
    /// let result = success.fold(
    ///     |e| format!("Error: {}", e),
    ///     |x| format!("Value: {}", x)
    /// );
    /// assert_eq!(result, "Value: 5");
    ///
    /// let failure = Validation::<i32, _>::failure("oops".to_string());
    /// let result = failure.fold(
    ///     |e| format!("Error: {}", e),
    ///     |x| format!("Value: {}", x)
    /// );
    /// assert_eq!(result, "Error: oops");
    /// ```
    #[inline]
    pub fn fold<R, F, G>(self, on_failure: F, on_success: G) -> R
    where
        F: FnOnce(E) -> R,
        G: FnOnce(T) -> R,
    {
        match self {
            Validation::Success(value) => on_success(value),
            Validation::Failure(error) => on_failure(error),
        }
    }

    /// Fold with success handler first.
    ///
    /// Alternative argument order for those who prefer success-first thinking.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(5);
    /// let msg = v.fold_success_first(
    ///     |x| format!("Got: {}", x),
    ///     |e| format!("Err: {}", e)
    /// );
    /// assert_eq!(msg, "Got: 5");
    /// ```
    #[inline]
    pub fn fold_success_first<R, F, G>(self, on_success: F, on_failure: G) -> R
    where
        F: FnOnce(T) -> R,
        G: FnOnce(E) -> R,
    {
        self.fold(on_failure, on_success)
    }

    /// Fold with an initial accumulator value.
    ///
    /// This is a generalized fold that takes a seed value and combines
    /// it with either the error or success value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let success = Validation::<i32, _>::success(5);
    /// let result = success.bifold(
    ///     10,
    ///     |acc, e: &str| acc + e.len() as i32,
    ///     |acc, x| acc + x
    /// );
    /// assert_eq!(result, 15); // 10 + 5
    /// ```
    #[inline]
    pub fn bifold<R, F, G>(self, seed: R, f: F, g: G) -> R
    where
        F: FnOnce(R, E) -> R,
        G: FnOnce(R, T) -> R,
    {
        match self {
            Validation::Success(value) => g(seed, value),
            Validation::Failure(error) => f(seed, error),
        }
    }

    // ========== unwrap variants ==========

    /// Unwrap the success value or compute it from the error.
    ///
    /// If Success, returns the value. If Failure, applies the function
    /// to the error to produce a value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let success = Validation::<_, String>::success(5);
    /// assert_eq!(success.unwrap_or_else(|e| e.len() as i32), 5);
    ///
    /// let failure = Validation::<i32, _>::failure("error".to_string());
    /// assert_eq!(failure.unwrap_or_else(|e| e.len() as i32), 5);
    /// ```
    #[inline]
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Validation::Success(value) => value,
            Validation::Failure(error) => f(error),
        }
    }

    /// Unwrap the success value or return a default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let success = Validation::<_, String>::success(5);
    /// assert_eq!(success.unwrap_or(0), 5);
    ///
    /// let failure = Validation::<i32, _>::failure("error".to_string());
    /// assert_eq!(failure.unwrap_or(0), 0);
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Validation::Success(value) => value,
            Validation::Failure(_) => default,
        }
    }

    /// Unwrap the success value or compute a default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let failure = Validation::<i32, _>::failure("error".to_string());
    /// assert_eq!(failure.unwrap_or_default(), 0);
    /// ```
    #[inline]
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Validation::Success(value) => value,
            Validation::Failure(_) => T::default(),
        }
    }

    /// Unwrap the error value, panicking if Success.
    ///
    /// # Panics
    ///
    /// Panics if this is a Success variant.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let failure = Validation::<i32, _>::failure("error");
    /// assert_eq!(failure.unwrap_err(), "error");
    /// ```
    #[inline]
    pub fn unwrap_err(self) -> E
    where
        T: std::fmt::Debug,
    {
        match self {
            Validation::Success(value) => panic!(
                "called `Validation::unwrap_err()` on a `Success` value: {:?}",
                value
            ),
            Validation::Failure(error) => error,
        }
    }

    /// Unwrap the error value with a custom panic message.
    #[inline]
    pub fn expect_err(self, msg: &str) -> E
    where
        T: std::fmt::Debug,
    {
        match self {
            Validation::Success(value) => panic!("{}: {:?}", msg, value),
            Validation::Failure(error) => error,
        }
    }
}

// ========== merge for same types ==========

impl<T> Validation<T, T> {
    /// Merge the Validation into its inner value.
    ///
    /// When the success and error types are the same, this extracts
    /// the inner value regardless of which variant it is.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let success: Validation<i32, i32> = Validation::success(5);
    /// assert_eq!(success.merge(), 5);
    ///
    /// let failure: Validation<i32, i32> = Validation::failure(3);
    /// assert_eq!(failure.merge(), 3);
    /// ```
    #[inline]
    pub fn merge(self) -> T {
        match self {
            Validation::Success(value) => value,
            Validation::Failure(error) => error,
        }
    }
}

// ========== Additional reference methods ==========

impl<T, E> Validation<T, E> {
    /// Apply bimap to references.
    ///
    /// Like `bimap` but works on references without consuming self.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(5);
    /// let result = v.bimap_ref(|e| e.len(), |x| *x * 2);
    /// assert_eq!(result, Validation::Success(10));
    /// // v is still usable
    /// ```
    #[inline]
    pub fn bimap_ref<U, E2, F, G>(&self, f: F, g: G) -> Validation<U, E2>
    where
        F: FnOnce(&E) -> E2,
        G: FnOnce(&T) -> U,
    {
        match self {
            Validation::Success(value) => Validation::Success(g(value)),
            Validation::Failure(error) => Validation::Failure(f(error)),
        }
    }

    /// Fold references without consuming self.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::Validation;
    ///
    /// let v = Validation::<_, String>::success(5);
    /// let result = v.fold_ref(|e| e.len(), |x| *x * 2);
    /// assert_eq!(result, 10);
    /// // v is still usable
    /// ```
    #[inline]
    pub fn fold_ref<R, F, G>(&self, on_failure: F, on_success: G) -> R
    where
        F: FnOnce(&E) -> R,
        G: FnOnce(&T) -> R,
    {
        match self {
            Validation::Success(value) => on_success(value),
            Validation::Failure(error) => on_failure(error),
        }
    }
}
```

### Bifunctor Laws (Documentation)

For completeness, document that `Validation` satisfies Bifunctor laws:

```rust
/// # Bifunctor Laws
///
/// `Validation` satisfies the Bifunctor laws:
///
/// 1. **Identity**: `v.bimap(id, id) == v`
/// 2. **Composition**: `v.bimap(f1, g1).bimap(f2, g2) == v.bimap(|e| f2(f1(e)), |x| g2(g1(x)))`
///
/// These laws ensure that `bimap` behaves predictably and can be composed.
```

## Dependencies

### Prerequisites
- None

### Affected Components
- `Validation<T, E>` in `src/validation/core.rs`

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod bimap_fold_tests {
    use super::*;

    // ========== bimap tests ==========

    #[test]
    fn test_bimap_on_success() {
        let v = Validation::<_, String>::success(5);
        let result = v.bimap(|e| e.len(), |x| x * 2);
        assert_eq!(result, Validation::Success(10));
    }

    #[test]
    fn test_bimap_on_failure() {
        let v = Validation::<i32, _>::failure("error".to_string());
        let result = v.bimap(|e| e.len(), |x| x * 2);
        assert_eq!(result, Validation::Failure(5));
    }

    #[test]
    fn test_bimap_type_transformation() {
        // Validation<i32, String> -> Validation<f64, usize>
        let v: Validation<i32, String> = Validation::success(5);
        let result: Validation<f64, usize> = v.bimap(|e| e.len(), |x| x as f64 * 2.0);
        assert_eq!(result, Validation::Success(10.0));
    }

    #[test]
    fn test_bimap_success_first() {
        let v = Validation::<_, String>::success(5);
        let result = v.bimap_success_first(|x| x * 2, |e| e.len());
        assert_eq!(result, Validation::Success(10));
    }

    // ========== fold tests ==========

    #[test]
    fn test_fold_on_success() {
        let v = Validation::<_, String>::success(5);
        let result = v.fold(|e| format!("Err: {}", e), |x| format!("Val: {}", x));
        assert_eq!(result, "Val: 5");
    }

    #[test]
    fn test_fold_on_failure() {
        let v = Validation::<i32, _>::failure("oops".to_string());
        let result = v.fold(|e| format!("Err: {}", e), |x| format!("Val: {}", x));
        assert_eq!(result, "Err: oops");
    }

    #[test]
    fn test_fold_to_different_type() {
        let v = Validation::<_, String>::success(5);
        let result: i32 = v.fold(|e| e.len() as i32, |x| x * 2);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_fold_success_first() {
        let v = Validation::<_, String>::success(5);
        let result = v.fold_success_first(|x| x * 2, |e| e.len() as i32);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_bifold() {
        let v = Validation::<_, String>::success(5);
        let result = v.bifold(10, |acc, e| acc + e.len() as i32, |acc, x| acc + x);
        assert_eq!(result, 15);

        let v = Validation::<i32, _>::failure("err".to_string());
        let result = v.bifold(10, |acc, e| acc + e.len() as i32, |acc, x| acc + x);
        assert_eq!(result, 13);
    }

    // ========== unwrap tests ==========

    #[test]
    fn test_unwrap_or_else_success() {
        let v = Validation::<_, String>::success(5);
        assert_eq!(v.unwrap_or_else(|e| e.len() as i32), 5);
    }

    #[test]
    fn test_unwrap_or_else_failure() {
        let v = Validation::<i32, _>::failure("error".to_string());
        assert_eq!(v.unwrap_or_else(|e| e.len() as i32), 5);
    }

    #[test]
    fn test_unwrap_or() {
        let success = Validation::<_, String>::success(5);
        assert_eq!(success.unwrap_or(0), 5);

        let failure = Validation::<i32, _>::failure("err".to_string());
        assert_eq!(failure.unwrap_or(0), 0);
    }

    #[test]
    fn test_unwrap_or_default() {
        let failure = Validation::<i32, _>::failure("err".to_string());
        assert_eq!(failure.unwrap_or_default(), 0);
    }

    // ========== merge tests ==========

    #[test]
    fn test_merge_success() {
        let v: Validation<i32, i32> = Validation::success(5);
        assert_eq!(v.merge(), 5);
    }

    #[test]
    fn test_merge_failure() {
        let v: Validation<i32, i32> = Validation::failure(3);
        assert_eq!(v.merge(), 3);
    }

    // ========== ref methods tests ==========

    #[test]
    fn test_bimap_ref() {
        let v = Validation::<_, String>::success(5);
        let result = v.bimap_ref(|e| e.len(), |x| *x * 2);
        assert_eq!(result, Validation::Success(10));
        // v is still usable
        assert!(v.is_success());
    }

    #[test]
    fn test_fold_ref() {
        let v = Validation::<_, String>::success(5);
        let result = v.fold_ref(|e| e.len() as i32, |x| *x * 2);
        assert_eq!(result, 10);
        // v is still usable
        assert!(v.is_success());
    }

    // ========== Bifunctor law tests ==========

    #[test]
    fn test_bifunctor_identity_law() {
        let v: Validation<i32, String> = Validation::success(5);
        let result = v.clone().bimap(|e| e, |x| x);
        assert_eq!(result, v);

        let v: Validation<i32, String> = Validation::failure("err".to_string());
        let result = v.clone().bimap(|e| e, |x| x);
        assert_eq!(result, v);
    }

    #[test]
    fn test_bifunctor_composition_law() {
        let f1 = |s: String| s.len();
        let f2 = |n: usize| n * 2;
        let g1 = |x: i32| x + 1;
        let g2 = |x: i32| x * 3;

        let v: Validation<i32, String> = Validation::success(5);

        // Two separate bimaps
        let left = v.clone().bimap(f1, g1).bimap(f2, g2);

        // Composed functions
        let right = v.bimap(|e| f2(f1(e)), |x| g2(g1(x)));

        assert_eq!(left, right);
    }

    // ========== integration tests ==========

    #[test]
    fn test_bimap_with_and_then() {
        let v = Validation::<_, Vec<String>>::success(5)
            .and_then(|x| {
                if x > 0 {
                    Validation::success(x * 2)
                } else {
                    Validation::failure(vec!["non-positive".to_string()])
                }
            })
            .bimap(|errs| errs.len(), |x| x + 1);

        assert_eq!(v, Validation::Success(11));
    }

    #[test]
    fn test_fold_as_final_operation() {
        let result = Validation::<_, Vec<String>>::success(5)
            .map(|x| x * 2)
            .fold(
                |errs| format!("{} errors", errs.len()),
                |x| format!("result: {}", x)
            );

        assert_eq!(result, "result: 10");
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
        fn prop_bifunctor_identity(x: i32) {
            let v: Validation<i32, i32> = Validation::success(x);
            prop_assert_eq!(v.clone().bimap(|e| e, |x| x), v);

            let v: Validation<i32, i32> = Validation::failure(x);
            prop_assert_eq!(v.clone().bimap(|e| e, |x| x), v);
        }

        #[test]
        fn prop_fold_exhaustive(x: i32, is_success: bool) {
            let v: Validation<i32, i32> = if is_success {
                Validation::success(x)
            } else {
                Validation::failure(x)
            };

            let result = v.fold(|e| e, |s| s);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn prop_merge_extracts_value(x: i32, is_success: bool) {
            let v: Validation<i32, i32> = if is_success {
                Validation::success(x)
            } else {
                Validation::failure(x)
            };

            prop_assert_eq!(v.merge(), x);
        }
    }
}
```

## Documentation Requirements

### Code Documentation

Full rustdoc on all methods with:
- Description of behavior
- Argument documentation
- Examples for both Success and Failure cases
- Links to related methods

### User Guide Addition

```markdown
## Working with Both Sides: bimap and fold

### bimap - Transform Both Variants

Transform success and error values in a single operation:

```rust
let result = validation.bimap(
    |error| wrap_error(error),     // Transform failure
    |value| transform_value(value) // Transform success
);
```

This is equivalent to but cleaner than:
```rust
let result = validation.map(transform_value).map_err(wrap_error);
```

### fold - Collapse to Single Value

Extract a value from either variant:

```rust
let message = validation.fold(
    |error| format!("Failed: {}", error),
    |value| format!("Success: {}", value)
);
```

### merge - When Types Are Same

When success and error types match, extract directly:

```rust
let v: Validation<String, String> = compute_something();
let result: String = v.merge(); // Works regardless of variant
```
```

## Implementation Notes

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| Failure handler first in `bimap`/`fold` | Matches Bifunctor convention (Left/Error first) |
| Provide `*_success_first` variants | Some prefer success-first thinking |
| `merge` only when `T == E` | Type safety - can't merge incompatible types |
| `bimap_ref`/`fold_ref` | Allows non-consuming operations |

### Naming Conventions

Following established FP terminology:
- `bimap` - Bifunctor's map over both type parameters
- `fold` - Catamorphism (collapse structure to value)
- `merge` - Extract from sum type when both sides are same

## Migration and Compatibility

- **Breaking changes**: None (additive)
- **New methods**: `bimap`, `bimap_success_first`, `fold`, `fold_success_first`, `bifold`, `merge`, `bimap_ref`, `fold_ref`, `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, `unwrap_err`, `expect_err`

---

*"When you need to work with both sides of the coin."*
