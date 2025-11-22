---
number: 001
title: Semigroup Trait for Error Accumulation
category: foundation
priority: critical
status: draft
dependencies: []
created: 2025-11-21
---

# Specification 001: Semigroup Trait for Error Accumulation

**Category**: foundation
**Priority**: critical
**Status**: draft
**Dependencies**: None

## Context

For validation accumulation to work, we need a way to combine multiple errors together. This is a fundamental algebraic structure called a Semigroup - a type that has an associative binary operation. This trait serves as the foundation for the Validation type's error accumulation capability.

Traditional Result types short-circuit on the first error, but validation scenarios (like form validation) benefit from collecting ALL errors at once. The Semigroup trait enables this by defining how to combine error values.

## Objective

Implement a `Semigroup` trait that provides a standard interface for combining values, enabling error accumulation in the Validation type. The implementation must be simple, extensible, and provide implementations for common Rust types.

## Requirements

### Functional Requirements

- Define a `Semigroup` trait with a `combine` method
- Implement Semigroup for `Vec<T>` (concatenation)
- Implement Semigroup for `String` (string concatenation)
- Implement Semigroup for tuples of Semigroups (component-wise combination)
- Provide clear documentation with examples
- Ensure associativity property holds for all implementations

### Non-Functional Requirements

- Zero-cost abstraction (inline where possible)
- Clear compiler errors when trait bounds not met
- Follow Rust naming conventions
- Comprehensive test coverage (>95%)
- Well-documented with examples

## Acceptance Criteria

- [ ] Semigroup trait defined in `src/semigroup.rs`
- [ ] Trait has single `combine(self, other: Self) -> Self` method
- [ ] Vec<T> implementation combines by extending
- [ ] String implementation combines by concatenation
- [ ] Tuple implementations for (T1, T2) through (T1, ..., T12)
- [ ] All implementations are associative (a.combine(b).combine(c) == a.combine(b.combine(c)))
- [ ] Property-based tests verify associativity
- [ ] Documentation includes usage examples
- [ ] Module is exported in lib.rs
- [ ] Zero compiler warnings

## Technical Details

### Implementation Approach

```rust
/// A type that can be combined associatively
pub trait Semigroup {
    /// Combine this value with another value associatively
    ///
    /// This operation must satisfy the associative property:
    /// `a.combine(b).combine(c) == a.combine(b.combine(c))`
    fn combine(self, other: Self) -> Self;
}
```

### Architecture Changes

- New module: `src/semigroup.rs`
- Export from `src/lib.rs`
- Re-export in `prelude` module

### Data Structures

No new data structures, only trait definition.

### APIs and Interfaces

```rust
// Core trait
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}

// Vec implementation
impl<T> Semigroup for Vec<T> {
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

// String implementation
impl Semigroup for String {
    fn combine(mut self, other: Self) -> Self {
        self.push_str(&other);
        self
    }
}

// Tuple implementations (macro-generated)
impl<T1: Semigroup, T2: Semigroup> Semigroup for (T1, T2) {
    fn combine(self, other: Self) -> Self {
        (self.0.combine(other.0), self.1.combine(other.1))
    }
}
// ... up to 12-tuples
```

## Dependencies

- **Prerequisites**: None (foundation specification)
- **Affected Components**: None (new module)
- **External Dependencies**: None

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_vec_semigroup() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5, 6];
    assert_eq!(v1.combine(v2), vec![1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_string_semigroup() {
    let s1 = "Hello, ".to_string();
    let s2 = "World!".to_string();
    assert_eq!(s1.combine(s2), "Hello, World!");
}

#[test]
fn test_tuple_semigroup() {
    let t1 = (vec![1], "a".to_string());
    let t2 = (vec![2], "b".to_string());
    assert_eq!(t1.combine(t2), (vec![1, 2], "ab".to_string()));
}
```

### Property Tests

```rust
// Using proptest or quickcheck
proptest! {
    #[test]
    fn associativity_vec(a: Vec<i32>, b: Vec<i32>, c: Vec<i32>) {
        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));
        assert_eq!(left, right);
    }
}
```

## Documentation Requirements

### Code Documentation

- Trait-level documentation explaining Semigroup concept
- Method documentation for `combine`
- Examples showing common use cases
- Note about associativity property

### User Documentation

- Add "Semigroup" section to README
- Explain why it's needed for validation
- Show how users can implement for custom types

### Architecture Updates

- Document in DESIGN.md as foundational trait

## Implementation Notes

### Macro for Tuple Implementations

Use a macro to generate tuple implementations:

```rust
macro_rules! impl_semigroup_tuple {
    ($($T:ident),+) => {
        impl<$($T: Semigroup),+> Semigroup for ($($T),+) {
            fn combine(self, other: Self) -> Self {
                #[allow(non_snake_case)]
                let ($($T),+) = self;
                #[allow(non_snake_case)]
                let ($(paste!{[<$T _other>]}),+) = other;
                (
                    $($T.combine(paste!{[<$T _other>]})),+
                )
            }
        }
    };
}
```

### Performance Considerations

- Vec::extend is efficient (single allocation when capacity sufficient)
- String::push_str is efficient (amortized O(1) per char)
- Tuple combination is zero-cost (inlined)

### Gotchas

- Semigroup takes `self` by value, not reference
- Users must clone if they need to keep original values
- Not all types have sensible Semigroup implementations

## Migration and Compatibility

No migration needed - this is a new feature with no breaking changes.

## Open Questions

1. Should we provide Semigroup impl for Option<T> where T: Semigroup?
   - `Some(a).combine(Some(b)) = Some(a.combine(b))`
   - `Some(a).combine(None) = Some(a)`
   - `None.combine(Some(b)) = Some(b)`

2. Should we provide `mconcat` helper for combining many values?
   ```rust
   fn mconcat<T: Semigroup>(values: Vec<T>) -> Option<T>
   ```

Decision: Defer to later spec if needed.
