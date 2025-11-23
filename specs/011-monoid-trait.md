---
number: 011
title: Monoid Trait Extension
category: foundation
priority: high
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 011: Monoid Trait Extension

**Category**: foundation
**Priority**: high
**Status**: draft
**Dependencies**: None (extends existing Semigroup trait)

## Context

Stillwater currently implements the `Semigroup` trait for error accumulation in `Validation`. A Semigroup defines an associative binary operation (`combine`), which allows combining two values of the same type.

A `Monoid` is a natural extension of `Semigroup` that adds an identity element (called `empty` or `mempty` in functional programming). This identity element has the property that combining it with any value returns that value unchanged:

```rust
// Monoid laws
a.combine(Monoid::empty()) == a
Monoid::empty().combine(a) == a
```

Monoids enable:
- Default/empty values for accumulation
- Folding operations without requiring an initial value
- Parallel reduction (split, process in parallel, combine results)
- Cleaner code when dealing with optional accumulation

## Objective

Extend Stillwater's algebraic type system with a `Monoid` trait that builds on `Semigroup`, providing identity elements for types that support them, enabling more powerful composition patterns.

## Requirements

### Functional Requirements

- Define `Monoid` trait that extends `Semigroup`
- Implement `Monoid` for all existing `Semigroup` types where identity exists:
  - `Vec<T>` (empty = `vec![]`)
  - `String` (empty = `""`)
  - `Option<T: Semigroup>` (empty = `None`)
  - Tuples of monoids (component-wise)
- Add numeric monoid wrappers:
  - `Sum<T: Add>` - addition with 0 as identity
  - `Product<T: Mul>` - multiplication with 1 as identity
  - `Max<T: Ord>` - maximum (requires bounded type)
  - `Min<T: Ord>` - minimum (requires bounded type)
- Provide convenience methods leveraging identity:
  - `fold_all(iter)` - fold without initial value
  - `reduce(iter)` - same as fold_all
- Update documentation and examples

### Non-Functional Requirements

- Zero runtime overhead (monomorphization)
- Type-safe identity guarantees
- Clear documentation of monoid laws
- Consistent with existing Semigroup design
- No breaking changes to existing APIs

## Acceptance Criteria

- [ ] `Monoid` trait defined with `empty()` method
- [ ] `Monoid` extends `Semigroup` trait
- [ ] Monoid laws documented in trait documentation
- [ ] Implemented for `Vec<T>`, `String`, `Option<T: Semigroup>`
- [ ] Tuple implementations (2-12 elements) via macro
- [ ] Numeric wrappers: `Sum`, `Product`, `Max`, `Min`
- [ ] Property-based tests verify monoid laws
- [ ] Integration with existing Validation/Effect types
- [ ] Documentation guide: `docs/guide/08-monoid.md`
- [ ] Examples demonstrating fold operations
- [ ] All tests pass
- [ ] Rustdoc examples compile and run

## Technical Details

### Implementation Approach

#### Trait Definition

```rust
/// A `Monoid` is a `Semigroup` with an identity element.
///
/// # Laws
///
/// For any value `a` of type `M` where `M: Monoid`:
///
/// ```text
/// a.combine(M::empty()) == a           (right identity)
/// M::empty().combine(a) == a           (left identity)
/// ```
///
/// Combined with `Semigroup` associativity:
///
/// ```text
/// a.combine(b).combine(c) == a.combine(b.combine(c))  (associativity)
/// ```
///
/// # Example
///
/// ```rust
/// use stillwater::{Monoid, Semigroup};
///
/// let v1 = vec![1, 2, 3];
/// let v2 = vec![4, 5];
/// let empty: Vec<i32> = Monoid::empty();
///
/// assert_eq!(v1.clone().combine(empty.clone()), v1);
/// assert_eq!(empty.combine(v1.clone()), v1);
/// ```
pub trait Monoid: Semigroup {
    /// The identity element for this monoid.
    ///
    /// Satisfies: `a.combine(Self::empty()) == a` and `Self::empty().combine(a) == a`
    fn empty() -> Self;
}
```

#### Implementations

```rust
// Vec monoid
impl<T> Monoid for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }
}

// String monoid
impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }
}

// Option monoid (lifts inner semigroup)
impl<T: Semigroup> Monoid for Option<T> {
    fn empty() -> Self {
        None
    }
}

// Tuple monoids (via macro for 2-12 elements)
impl<T1: Monoid, T2: Monoid> Monoid for (T1, T2) {
    fn empty() -> Self {
        (T1::empty(), T2::empty())
    }
}
```

#### Numeric Wrappers

```rust
/// Monoid for numeric types under addition.
///
/// Identity: 0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sum<T>(pub T);

impl<T: Add<Output = T> + Default> Semigroup for Sum<T> {
    fn combine(self, other: Self) -> Self {
        Sum(self.0 + other.0)
    }
}

impl<T: Add<Output = T> + Default> Monoid for Sum<T> {
    fn empty() -> Self {
        Sum(T::default())
    }
}

/// Monoid for numeric types under multiplication.
///
/// Identity: 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Product<T>(pub T);

impl<T: Mul<Output = T> + One> Semigroup for Product<T> {
    fn combine(self, other: Self) -> Self {
        Product(self.0 * other.0)
    }
}

impl<T: Mul<Output = T> + One> Monoid for Product<T> {
    fn empty() -> Self {
        Product(T::one())
    }
}

/// Monoid for ordered types under maximum.
///
/// Identity: Minimum bound
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Max<T>(pub T);

/// Monoid for ordered types under minimum.
///
/// Identity: Maximum bound
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Min<T>(pub T);
```

#### Utility Functions

```rust
/// Fold an iterator using the Monoid instance, starting with `empty()`.
///
/// # Example
///
/// ```rust
/// use stillwater::monoid::fold_all;
///
/// let numbers = vec![
///     vec![1, 2],
///     vec![3, 4],
///     vec![5],
/// ];
///
/// let result: Vec<i32> = fold_all(numbers);
/// assert_eq!(result, vec![1, 2, 3, 4, 5]);
/// ```
pub fn fold_all<M, I>(iter: I) -> M
where
    M: Monoid,
    I: IntoIterator<Item = M>,
{
    iter.into_iter().fold(M::empty(), |acc, x| acc.combine(x))
}
```

### Architecture Changes

- New module: `src/monoid.rs`
- Re-export from `src/lib.rs`
- Update `src/semigroup.rs` to work with Monoid
- Add numeric wrapper types in monoid module

### Data Structures

```rust
// src/monoid.rs structure
pub trait Monoid: Semigroup {
    fn empty() -> Self;
}

pub struct Sum<T>(pub T);
pub struct Product<T>(pub T);
pub struct Max<T>(pub T);
pub struct Min<T>(pub T);

pub fn fold_all<M, I>(iter: I) -> M { /* ... */ }
pub fn reduce<M, I>(iter: I) -> M { /* ... */ }  // alias
```

### APIs and Interfaces

Integration with existing types:

```rust
// Validation can now use fold_all for combining
let validations = vec![val1, val2, val3];
let result = fold_all(validations);

// Effect can use monoid for combining results
effects.map(|results| fold_all(results))
```

## Dependencies

- **Prerequisites**: None (extends existing Semigroup)
- **Affected Components**:
  - `src/semigroup.rs` (Monoid extends Semigroup)
  - `src/validation.rs` (can use Monoid utilities)
  - Documentation updates
- **External Dependencies**: None (uses std only)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Law: right identity
    #[test]
    fn test_right_identity() {
        let v = vec![1, 2, 3];
        let empty: Vec<i32> = Monoid::empty();
        assert_eq!(v.clone().combine(empty), v);
    }

    // Law: left identity
    #[test]
    fn test_left_identity() {
        let v = vec![1, 2, 3];
        let empty: Vec<i32> = Monoid::empty();
        assert_eq!(empty.combine(v.clone()), v);
    }

    // fold_all test
    #[test]
    fn test_fold_all() {
        let vecs = vec![vec![1], vec![2, 3], vec![4]];
        let result = fold_all(vecs);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    // Sum monoid
    #[test]
    fn test_sum_monoid() {
        let nums = vec![Sum(1), Sum(2), Sum(3)];
        let result = fold_all(nums);
        assert_eq!(result, Sum(6));
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_right_identity(v: Vec<i32>) {
        let empty: Vec<i32> = Monoid::empty();
        prop_assert_eq!(v.clone().combine(empty), v);
    }

    #[test]
    fn prop_left_identity(v: Vec<i32>) {
        let empty: Vec<i32> = Monoid::empty();
        prop_assert_eq!(empty.combine(v.clone()), v);
    }

    #[test]
    fn prop_associativity(a: Vec<i32>, b: Vec<i32>, c: Vec<i32>) {
        prop_assert_eq!(
            a.clone().combine(b.clone()).combine(c.clone()),
            a.combine(b.combine(c))
        );
    }
}
```

### Integration Tests

Test interaction with Validation and Effect:

```rust
#[test]
fn test_validation_with_monoid() {
    let vals = vec![
        Validation::success(vec![1]),
        Validation::success(vec![2, 3]),
        Validation::success(vec![4]),
    ];

    let result = fold_all(vals);
    assert_eq!(result, Validation::success(vec![1, 2, 3, 4]));
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for `Monoid` trait
- Document monoid laws with examples
- Rustdoc examples for all numeric wrappers
- Document relationship to Semigroup

### User Documentation

- New guide: `docs/guide/08-monoid.md`
- Update README with Monoid example
- Add to PHILOSOPHY.md explanation
- FAQ entry: "When to use Monoid vs Semigroup?"

### Architecture Updates

- Update DESIGN.md with Monoid pattern
- Document numeric wrapper types
- Explain fold_all utility

## Implementation Notes

### Design Decisions

**Why extend Semigroup?**
- Natural hierarchy: all Monoids are Semigroups
- Allows using Monoid anywhere Semigroup is expected
- Mirrors mathematical structure

**Why numeric wrappers?**
- Rust primitives can't implement foreign traits
- Multiple possible monoids for same type (Sum vs Product)
- Explicit choice prevents ambiguity

**Why fold_all instead of just using Iterator::fold?**
- Leverages type system for identity element
- More ergonomic for monoid types
- Clearer intent in code

### Gotchas

- Not all types with Semigroup have Monoid (e.g., `NonEmpty` lists)
- Be careful with numeric overflow in Sum/Product
- Max/Min require bounded types (can't have identity for unbounded)

### Best Practices

- Use Monoid when you need default/empty values
- Use fold_all for cleaner reduction code
- Prefer type-safe wrappers (Sum) over raw primitives
- Document which monoid you're using (addition vs multiplication)

## Migration and Compatibility

### Breaking Changes

None - this is a pure addition.

### Compatibility

- Fully backward compatible
- Existing Semigroup code works unchanged
- Opt-in usage of Monoid features

### Migration Path

No migration needed - existing code continues to work.

Users can opt-in to Monoid features:

```rust
// Before (still works)
let result = vec![val1, val2].into_iter()
    .fold(initial, |acc, v| acc.combine(v));

// After (more ergonomic with Monoid)
let result = fold_all(vec![val1, val2]);
```

## Related Patterns

- **Semigroup**: Monoid builds on this
- **Foldable**: Monoid enables folding without initial value
- **Parallel reduction**: Monoid identity enables parallel splits

## Success Metrics

- All monoid law tests pass
- Property-based tests verify laws for all implementations
- Zero performance regression vs manual implementation
- Documentation is clear and comprehensive
- User feedback is positive

## Future Enhancements

Potential additions in later versions:

- `All` and `Any` monoids for boolean logic
- `First` and `Last` monoids for precedence
- `Endo` monoid for function composition
- Integration with parallel iterators (rayon)
- Monoid instances for more std types (HashMap, BTreeMap)
