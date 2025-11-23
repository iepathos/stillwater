---
number: 014
title: Extended Semigroup Implementations
category: foundation
priority: medium
status: draft
dependencies: [011]
created: 2025-11-22
---

# Specification 014: Extended Semigroup Implementations

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: Spec 011 (Monoid trait)

## Context

Stillwater currently implements `Semigroup` for basic types (`Vec<T>`, `String`, tuples). However, many standard Rust types have natural semigroup structures that would be useful for error accumulation and data combining:

- **`HashMap<K, V>`** - Merge maps (various strategies: left-biased, right-biased, or combine values)
- **`HashSet<T>`** - Union, intersection, or difference
- **`BTreeMap<K, V>`** - Same as HashMap but ordered
- **`BTreeSet<T>`** - Same as HashSet but ordered
- **`Option<T: Semigroup>`** - Lift semigroup operation to Option

These implementations would enable more powerful composition patterns, especially for configuration merging, aggregation, and accumulation.

## Objective

Extend Stillwater's Semigroup trait with implementations for standard Rust collection types, enabling natural composition of maps, sets, and optional values.

## Requirements

### Functional Requirements

- Implement `Semigroup` for `HashMap<K, V: Semigroup>`
- Implement `Semigroup` for `HashSet<T>`
- Implement `Semigroup` for `BTreeMap<K, V: Semigroup>`
- Implement `Semigroup` for `BTreeSet<T>`
- Implement `Semigroup` for `Option<T: Semigroup>`
- Provide wrapper types for alternative semantics:
  - `First<T>` - keep first value (left-biased)
  - `Last<T>` - keep last value (right-biased)
  - `Union<Set>` - set union (default for sets)
  - `Intersection<Set>` - set intersection
- Document semantics clearly
- Comprehensive tests

### Non-Functional Requirements

- Zero overhead compared to manual implementations
- Type-safe combination rules
- Clear, predictable semantics
- Integration with existing Semigroup ecosystem

## Acceptance Criteria

- [ ] `Semigroup` for `HashMap<K, V: Semigroup>` (merge, combine values)
- [ ] `Semigroup` for `HashSet<T>` (union)
- [ ] `Semigroup` for `BTreeMap<K, V: Semigroup>` (merge, combine values)
- [ ] `Semigroup` for `BTreeSet<T>` (union)
- [ ] `Semigroup` for `Option<T: Semigroup>` (Some + Some = Some(combined))
- [ ] Wrapper types: `First<T>`, `Last<T>`
- [ ] Monoid instances where appropriate
- [ ] Property-based tests verify associativity
- [ ] Documentation with practical examples
- [ ] All tests pass

## Technical Details

### Implementation Approach

#### HashMap Semigroup

```rust
use std::collections::HashMap;
use std::hash::Hash;

/// Semigroup for HashMap that merges maps, combining values with the same key.
///
/// When keys conflict, values are combined using their Semigroup instance.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use stillwater::Semigroup;
///
/// let mut map1 = HashMap::new();
/// map1.insert("errors", vec!["error1"]);
/// map1.insert("warnings", vec!["warn1"]);
///
/// let mut map2 = HashMap::new();
/// map2.insert("errors", vec!["error2"]);
/// map2.insert("info", vec!["info1"]);
///
/// let combined = map1.combine(map2);
/// // Result:
/// // {
/// //   "errors": ["error1", "error2"],  // Combined
/// //   "warnings": ["warn1"],            // From map1
/// //   "info": ["info1"]                 // From map2
/// // }
/// ```
impl<K, V> Semigroup for HashMap<K, V>
where
    K: Eq + Hash,
    V: Semigroup,
{
    fn combine(mut self, other: Self) -> Self {
        for (key, value) in other {
            self.entry(key)
                .and_modify(|existing| {
                    *existing = existing.clone().combine(value.clone());
                })
                .or_insert(value);
        }
        self
    }
}

impl<K, V> Monoid for HashMap<K, V>
where
    K: Eq + Hash,
    V: Semigroup,
{
    fn empty() -> Self {
        HashMap::new()
    }
}
```

#### HashSet Semigroup

```rust
use std::collections::HashSet;

/// Semigroup for HashSet using union.
///
/// # Example
///
/// ```rust
/// use std::collections::HashSet;
/// use stillwater::Semigroup;
///
/// let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
/// let set2: HashSet<_> = [3, 4, 5].iter().cloned().collect();
///
/// let combined = set1.combine(set2);
/// assert_eq!(combined.len(), 5); // {1, 2, 3, 4, 5}
/// ```
impl<T> Semigroup for HashSet<T>
where
    T: Eq + Hash,
{
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl<T> Monoid for HashSet<T>
where
    T: Eq + Hash,
{
    fn empty() -> Self {
        HashSet::new()
    }
}
```

#### BTreeMap/BTreeSet

```rust
use std::collections::{BTreeMap, BTreeSet};

/// Semigroup for BTreeMap (same semantics as HashMap).
impl<K, V> Semigroup for BTreeMap<K, V>
where
    K: Ord,
    V: Semigroup,
{
    fn combine(mut self, other: Self) -> Self {
        for (key, value) in other {
            self.entry(key)
                .and_modify(|existing| {
                    *existing = existing.clone().combine(value.clone());
                })
                .or_insert(value);
        }
        self
    }
}

impl<K, V> Monoid for BTreeMap<K, V>
where
    K: Ord,
    V: Semigroup,
{
    fn empty() -> Self {
        BTreeMap::new()
    }
}

/// Semigroup for BTreeSet using union.
impl<T> Semigroup for BTreeSet<T>
where
    T: Ord,
{
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl<T> Monoid for BTreeSet<T>
where
    T: Ord,
{
    fn empty() -> Self {
        BTreeSet::new()
    }
}
```

#### Option Semigroup

```rust
/// Semigroup for Option that lifts the inner Semigroup.
///
/// - `Some(a).combine(Some(b))` = `Some(a.combine(b))`
/// - `Some(a).combine(None)` = `Some(a)`
/// - `None.combine(Some(b))` = `Some(b)`
/// - `None.combine(None)` = `None`
///
/// # Example
///
/// ```rust
/// use stillwater::Semigroup;
///
/// let opt1 = Some(vec![1, 2]);
/// let opt2 = Some(vec![3, 4]);
/// let result = opt1.combine(opt2);
/// assert_eq!(result, Some(vec![1, 2, 3, 4]));
///
/// let none: Option<Vec<i32>> = None;
/// let some = Some(vec![1, 2]);
/// assert_eq!(none.combine(some.clone()), some);
/// ```
impl<T> Semigroup for Option<T>
where
    T: Semigroup,
{
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.combine(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }
}

impl<T> Monoid for Option<T>
where
    T: Semigroup,
{
    fn empty() -> Self {
        None
    }
}
```

#### Wrapper Types

```rust
/// Wrapper type that keeps the first value (left-biased).
///
/// # Example
///
/// ```rust
/// use stillwater::{First, Semigroup};
///
/// let first = First(1).combine(First(2));
/// assert_eq!(first.0, 1); // Keeps first
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct First<T>(pub T);

impl<T> Semigroup for First<T> {
    fn combine(self, _other: Self) -> Self {
        self // Always keep first
    }
}

/// Wrapper type that keeps the last value (right-biased).
///
/// # Example
///
/// ```rust
/// use stillwater::{Last, Semigroup};
///
/// let last = Last(1).combine(Last(2));
/// assert_eq!(last.0, 2); // Keeps last
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Last<T>(pub T);

impl<T> Semigroup for Last<T> {
    fn combine(self, other: Self) -> Self {
        other // Always keep last
    }
}

/// Wrapper for set intersection (alternative to union).
///
/// # Example
///
/// ```rust
/// use std::collections::HashSet;
/// use stillwater::{Intersection, Semigroup};
///
/// let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
/// let set2: HashSet<_> = [2, 3, 4].iter().cloned().collect();
///
/// let i1 = Intersection(set1);
/// let i2 = Intersection(set2);
/// let result = i1.combine(i2);
/// assert_eq!(result.0, [2, 3].iter().cloned().collect()); // Intersection
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intersection<S>(pub S);

impl<T> Semigroup for Intersection<HashSet<T>>
where
    T: Eq + Hash + Clone,
{
    fn combine(self, other: Self) -> Self {
        Intersection(self.0.intersection(&other.0).cloned().collect())
    }
}
```

### Architecture Changes

- Update `src/semigroup.rs` with new implementations
- Add wrapper types module or include in semigroup.rs
- Update prelude to re-export wrappers

### Integration Patterns

#### Configuration Merging

```rust
use std::collections::HashMap;

#[derive(Clone)]
struct Config {
    settings: HashMap<String, String>,
    features: HashSet<String>,
}

impl Semigroup for Config {
    fn combine(self, other: Self) -> Self {
        Config {
            settings: self.settings.combine(other.settings),
            features: self.features.combine(other.features),
        }
    }
}

// Merge configs from multiple sources
let default_config = load_default_config();
let user_config = load_user_config();
let env_config = load_env_config();

let final_config = default_config
    .combine(user_config)
    .combine(env_config);
```

#### Error Aggregation

```rust
use std::collections::HashMap;

type ErrorsByType = HashMap<String, Vec<String>>;

let errors1: ErrorsByType = [
    ("validation".into(), vec!["Invalid email".into()]),
].iter().cloned().collect();

let errors2: ErrorsByType = [
    ("validation".into(), vec!["Invalid age".into()]),
    ("permission".into(), vec!["Unauthorized".into()]),
].iter().cloned().collect();

let all_errors = errors1.combine(errors2);
// {
//   "validation": ["Invalid email", "Invalid age"],
//   "permission": ["Unauthorized"]
// }
```

## Dependencies

- **Prerequisites**: Spec 011 (Monoid trait)
- **Affected Components**:
  - `src/semigroup.rs`
  - Documentation
- **External Dependencies**: std only

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_hashmap_combine() {
        let mut map1 = HashMap::new();
        map1.insert("a", vec![1, 2]);

        let mut map2 = HashMap::new();
        map2.insert("a", vec![3, 4]);
        map2.insert("b", vec![5]);

        let result = map1.combine(map2);
        assert_eq!(result.get("a"), Some(&vec![1, 2, 3, 4]));
        assert_eq!(result.get("b"), Some(&vec![5]));
    }

    #[test]
    fn test_hashset_union() {
        let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
        let set2: HashSet<_> = [3, 4, 5].iter().cloned().collect();

        let result = set1.combine(set2);
        assert_eq!(result.len(), 5);
        assert!(result.contains(&1));
        assert!(result.contains(&5));
    }

    #[test]
    fn test_option_semigroup() {
        let opt1 = Some(vec![1, 2]);
        let opt2 = Some(vec![3, 4]);
        assert_eq!(opt1.combine(opt2), Some(vec![1, 2, 3, 4]));

        let none: Option<Vec<i32>> = None;
        let some = Some(vec![1]);
        assert_eq!(none.clone().combine(some.clone()), some);
        assert_eq!(some.clone().combine(none), some);
    }

    #[test]
    fn test_first_last() {
        assert_eq!(First(1).combine(First(2)), First(1));
        assert_eq!(Last(1).combine(Last(2)), Last(2));
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_hashmap_associative(
        a: HashMap<String, Vec<i32>>,
        b: HashMap<String, Vec<i32>>,
        c: HashMap<String, Vec<i32>>,
    ) {
        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn prop_hashset_associative(
        a: HashSet<i32>,
        b: HashSet<i32>,
        c: HashSet<i32>,
    ) {
        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));
        prop_assert_eq!(left, right);
    }
}
```

## Documentation Requirements

### Code Documentation

- Rustdoc for each implementation
- Examples showing practical use cases
- Document combining semantics clearly
- Explain wrapper types

### User Documentation

- Update `docs/guide/01-semigroup.md` with new implementations
- Add configuration merging example
- Add error aggregation example
- FAQ: "How to merge HashMaps?"

## Implementation Notes

### Design Decisions

**Why combine values in HashMap instead of keeping first/last?**
- Most useful for accumulation (errors, configs)
- Use `First`/`Last` wrappers for other semantics
- Follows Haskell's `Data.Map` behavior

**Why union for sets, not intersection?**
- Union is associative monoid (intersection isn't)
- Union is more commonly needed
- `Intersection` wrapper available for other use case

**Why clone in Option combine?**
- Necessary for combining values
- Could add move-only version later

## Migration and Compatibility

- Fully backward compatible
- Pure additions, no breaking changes

## Success Metrics

- Associativity verified by property tests
- Zero performance regression
- Positive user feedback

## Future Enhancements

- `Union`/`Intersection` wrappers for all set types
- More efficient combine strategies (no clone)
- Integration with rayon for parallel combining
