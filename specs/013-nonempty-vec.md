---
number: 013
title: NonEmptyVec Data Structure
category: foundation
priority: medium
status: draft
dependencies: []
created: 2025-11-22
---

# Specification 013: NonEmptyVec Data Structure

**Category**: foundation
**Priority**: medium
**Status**: draft
**Dependencies**: None

## Context

In functional programming, it's common to need a vector that is guaranteed to have at least one element. This provides type-level guarantees that prevent runtime errors when operations assume a non-empty collection.

Use cases in Stillwater:
- **Validation errors**: When a `Validation` fails, there's always at least one error
- **Aggregations**: Some operations (like `head()`, `max()`, `min()`) require non-empty data
- **Type safety**: Prevent `None`/`panic!` in operations that need elements

Currently, `Validation<T, Vec<E>>` can fail with an empty error vector, which shouldn't be possible. `NonEmptyVec<E>` makes this impossible at the type level.

## Objective

Implement a `NonEmptyVec<T>` data structure that guarantees at least one element exists, providing type-safe operations that don't require Option returns or panic.

## Requirements

### Functional Requirements

- Define `NonEmptyVec<T>` struct with at least one element guaranteed
- Provide safe construction methods
- Implement standard collection operations:
  - `head()` / `first()` - always succeeds
  - `tail()` - returns `Vec<T>` (may be empty)
  - `last()` - always succeeds
  - `push()`, `pop()` - safe operations
  - `map()`, `filter()` (returns `Vec<T>`), `iter()`
- Implement `FromIterator` with fallible conversion
- Provide infallible `from_vec_unchecked()` for trusted sources
- Integrate with `Validation` error type
- Implement standard traits: `Debug`, `Clone`, `PartialEq`, `IntoIterator`

### Non-Functional Requirements

- Zero-cost abstraction (same performance as Vec)
- Clear API preventing misuse
- Comprehensive documentation
- Interoperable with Vec

## Acceptance Criteria

- [ ] `NonEmptyVec<T>` struct defined
- [ ] Construction methods: `new()`, `from_vec()`, `from_vec_unchecked()`
- [ ] Safe accessor methods: `head()`, `tail()`, `last()`
- [ ] Collection operations: `push()`, `pop()`, `len()`, `iter()`
- [ ] Transformation methods: `map()`, `filter()`
- [ ] Implements `FromIterator` with fallible conversion
- [ ] Implements `IntoIterator`, `Debug`, `Clone`, `PartialEq`, `Eq`
- [ ] Semigroup and Monoid instances
- [ ] Integration with `Validation` type
- [ ] Comprehensive tests
- [ ] Documentation with examples
- [ ] All tests pass

## Technical Details

### Implementation Approach

#### Core Structure

```rust
/// A non-empty vector guaranteed to contain at least one element.
///
/// This type provides type-level guarantees that operations like `head()`,
/// `max()`, and `min()` will always succeed without returning `Option`.
///
/// # Example
///
/// ```rust
/// use stillwater::NonEmptyVec;
///
/// let nev = NonEmptyVec::new(1, vec![2, 3, 4]);
/// assert_eq!(nev.head(), &1);
/// assert_eq!(nev.tail(), &[2, 3, 4]);
/// assert_eq!(nev.len(), 4);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    head: T,
    tail: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    /// Create a new non-empty vector with a head element and tail.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.len(), 3);
    /// ```
    pub fn new(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }

    /// Create a non-empty vector from a single element.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::singleton(42);
    /// assert_eq!(nev.len(), 1);
    /// assert_eq!(nev.head(), &42);
    /// ```
    pub fn singleton(value: T) -> Self {
        Self::new(value, Vec::new())
    }

    /// Try to create a non-empty vector from a `Vec`.
    ///
    /// Returns `None` if the vector is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::from_vec(vec![1, 2, 3]).unwrap();
    /// assert_eq!(nev.len(), 3);
    ///
    /// let empty = NonEmptyVec::from_vec(Vec::<i32>::new());
    /// assert!(empty.is_none());
    /// ```
    pub fn from_vec(mut vec: Vec<T>) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            let head = vec.remove(0);
            Some(Self::new(head, vec))
        }
    }

    /// Create a non-empty vector from a `Vec` without checking.
    ///
    /// # Safety
    ///
    /// The caller must ensure the vector is non-empty. Panics if empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::from_vec_unchecked(vec![1, 2, 3]);
    /// assert_eq!(nev.len(), 3);
    /// ```
    ///
    /// ```should_panic
    /// let nev = NonEmptyVec::from_vec_unchecked(Vec::<i32>::new()); // panics
    /// ```
    pub fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self::from_vec(vec).expect("NonEmptyVec::from_vec_unchecked called on empty Vec")
    }

    /// Get the first element (always succeeds).
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.head(), &1);
    /// ```
    pub fn head(&self) -> &T {
        &self.head
    }

    /// Get the tail (all elements except the first).
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.tail(), &[2, 3]);
    /// ```
    pub fn tail(&self) -> &[T] {
        &self.tail
    }

    /// Get the last element (always succeeds).
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.last(), &3);
    ///
    /// let single = NonEmptyVec::singleton(42);
    /// assert_eq!(single.last(), &42);
    /// ```
    pub fn last(&self) -> &T {
        self.tail.last().unwrap_or(&self.head)
    }

    /// Get the number of elements.
    ///
    /// Always >= 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    /// Push an element to the end.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut nev = NonEmptyVec::singleton(1);
    /// nev.push(2);
    /// nev.push(3);
    /// assert_eq!(nev.len(), 3);
    /// ```
    pub fn push(&mut self, value: T) {
        self.tail.push(value);
    }

    /// Pop an element from the end.
    ///
    /// Returns `None` if there's only one element (the head).
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.pop(), Some(3));
    /// assert_eq!(nev.pop(), Some(2));
    /// assert_eq!(nev.pop(), None); // Can't remove head
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        self.tail.pop()
    }

    /// Map a function over all elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// let doubled = nev.map(|x| x * 2);
    /// assert_eq!(doubled.head(), &2);
    /// assert_eq!(doubled.tail(), &[4, 6]);
    /// ```
    pub fn map<U, F>(self, mut f: F) -> NonEmptyVec<U>
    where
        F: FnMut(T) -> U,
    {
        let head = f(self.head);
        let tail = self.tail.into_iter().map(f).collect();
        NonEmptyVec::new(head, tail)
    }

    /// Filter elements (may return empty Vec).
    ///
    /// Since filtering might remove all elements, this returns `Vec<T>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3, 4]);
    /// let evens = nev.filter(|x| x % 2 == 0);
    /// assert_eq!(evens, vec![2, 4]);
    ///
    /// let none = NonEmptyVec::singleton(1).filter(|x| x % 2 == 0);
    /// assert_eq!(none, vec![]);
    /// ```
    pub fn filter<F>(self, mut predicate: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        let mut result = Vec::new();
        if predicate(&self.head) {
            result.push(self.head);
        }
        result.extend(self.tail.into_iter().filter(predicate));
        result
    }

    /// Convert to a regular `Vec`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// let vec = nev.into_vec();
    /// assert_eq!(vec, vec![1, 2, 3]);
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        let mut vec = vec![self.head];
        vec.extend(self.tail);
        vec
    }

    /// Iterate over all elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// let sum: i32 = nev.iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }
}
```

### Trait Implementations

```rust
// Semigroup: concatenation
impl<T> Semigroup for NonEmptyVec<T> {
    fn combine(mut self, other: Self) -> Self {
        self.tail.push(other.head);
        self.tail.extend(other.tail);
        self
    }
}

// IntoIterator
impl<T> IntoIterator for NonEmptyVec<T> {
    type Item = T;
    type IntoIter = std::iter::Chain<std::iter::Once<T>, std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.head).chain(self.tail.into_iter())
    }
}

// FromIterator (fallible)
impl<T> FromIterator<T> for Option<NonEmptyVec<T>> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        NonEmptyVec::from_vec(vec)
    }
}

// Index
impl<T> std::ops::Index<usize> for NonEmptyVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.head
        } else {
            &self.tail[index - 1]
        }
    }
}
```

### Integration with Validation

```rust
// Update Validation to use NonEmptyVec for errors
impl<T, E> Validation<T, NonEmptyVec<E>> {
    /// Create a failure with a single error.
    pub fn fail(error: E) -> Self {
        Validation::failure(NonEmptyVec::singleton(error))
    }

    /// Combine validations, accumulating errors in NonEmptyVec.
    pub fn all<I>(validations: I) -> Validation<Vec<T>, NonEmptyVec<E>>
    where
        I: IntoIterator<Item = Validation<T, NonEmptyVec<E>>>,
    {
        // Implementation that accumulates errors
    }
}
```

## Dependencies

- **Prerequisites**: None
- **Affected Components**:
  - New module: `src/nonempty.rs`
  - Integration with `src/validation.rs`
  - Documentation updates
- **External Dependencies**: None (std only)

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton() {
        let nev = NonEmptyVec::singleton(42);
        assert_eq!(nev.head(), &42);
        assert_eq!(nev.tail(), &[]);
        assert_eq!(nev.len(), 1);
    }

    #[test]
    fn test_from_vec() {
        let nev = NonEmptyVec::from_vec(vec![1, 2, 3]).unwrap();
        assert_eq!(nev.head(), &1);
        assert_eq!(nev.tail(), &[2, 3]);

        let empty = NonEmptyVec::from_vec(Vec::<i32>::new());
        assert!(empty.is_none());
    }

    #[test]
    fn test_push_pop() {
        let mut nev = NonEmptyVec::singleton(1);
        nev.push(2);
        nev.push(3);
        assert_eq!(nev.len(), 3);

        assert_eq!(nev.pop(), Some(3));
        assert_eq!(nev.pop(), Some(2));
        assert_eq!(nev.pop(), None);
        assert_eq!(nev.len(), 1);
    }

    #[test]
    fn test_map() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        let doubled = nev.map(|x| x * 2);
        assert_eq!(doubled.into_vec(), vec![2, 4, 6]);
    }

    #[test]
    fn test_filter() {
        let nev = NonEmptyVec::new(1, vec![2, 3, 4]);
        let evens = nev.filter(|x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
    }

    #[test]
    fn test_semigroup() {
        let nev1 = NonEmptyVec::new(1, vec![2]);
        let nev2 = NonEmptyVec::new(3, vec![4]);
        let combined = nev1.combine(nev2);
        assert_eq!(combined.into_vec(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_into_iter() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        let vec: Vec<_> = nev.into_iter().collect();
        assert_eq!(vec, vec![1, 2, 3]);
    }
}
```

## Documentation Requirements

### Code Documentation

- Comprehensive rustdoc for all methods
- Examples for each operation
- Document why NonEmptyVec vs Vec
- Explain type safety guarantees

### User Documentation

- New guide: `docs/guide/10-nonempty-collections.md`
- Update Validation guide with NonEmptyVec usage
- Add to README examples
- FAQ: "When to use NonEmptyVec?"

## Implementation Notes

### Design Decisions

**Why head + tail instead of Vec wrapper?**
- Makes "at least one" explicit in structure
- Efficient head access (no indexing)
- Clear separation of guaranteed vs optional elements

**Why no Monoid instance?**
- No sensible empty value (contradicts "non-empty")
- Only Semigroup (concatenation)

**Why filter returns Vec?**
- Filtering might remove all elements
- Type system enforces thinking about this case
- Alternative: `filter_ne()` that returns `Option<NonEmptyVec<T>>`

## Migration and Compatibility

### Breaking Changes

None - pure addition.

### Compatibility

- Fully backward compatible
- Opt-in usage where type safety is desired

## Success Metrics

- Zero runtime overhead vs manual Vec + bool check
- Positive user feedback on type safety
- Reduced runtime errors in validation code

## Future Enhancements

- `NonEmpty<Vec<T>>` trait for generic non-empty collections
- `filter_ne()` returning `Option<NonEmptyVec<T>>`
- Additional methods: `split_first()`, `split_last()`, etc.
- NonEmptySet, NonEmptyMap variants
