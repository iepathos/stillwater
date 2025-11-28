//! Collection predicates
//!
//! This module provides common predicates for collection validation.

use super::combinators::Predicate;

/// Predicate that checks if a collection is empty.
#[derive(Clone, Copy, Default, Debug)]
pub struct IsEmpty;

impl<T> Predicate<Vec<T>> for IsEmpty {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.is_empty()
    }
}

impl<T> Predicate<[T]> for IsEmpty {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.is_empty()
    }
}

/// Create a predicate that checks if a collection is empty.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(is_empty().check(&vec![] as &Vec<i32>));
/// assert!(!is_empty().check(&vec![1, 2, 3]));
/// ```
pub fn is_empty() -> IsEmpty {
    IsEmpty
}

/// Predicate that checks if a collection is not empty.
#[derive(Clone, Copy, Default, Debug)]
pub struct IsNotEmpty;

impl<T> Predicate<Vec<T>> for IsNotEmpty {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        !value.is_empty()
    }
}

impl<T> Predicate<[T]> for IsNotEmpty {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        !value.is_empty()
    }
}

/// Create a predicate that checks if a collection is not empty.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(is_not_empty().check(&vec![1, 2, 3]));
/// assert!(!is_not_empty().check(&vec![] as &Vec<i32>));
/// ```
pub fn is_not_empty() -> IsNotEmpty {
    IsNotEmpty
}

/// Predicate that checks collection length equals expected.
#[derive(Clone, Copy, Debug)]
pub struct HasLen {
    expected: usize,
}

impl<T> Predicate<Vec<T>> for HasLen {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.len() == self.expected
    }
}

impl<T> Predicate<[T]> for HasLen {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.len() == self.expected
    }
}

/// Create a predicate that checks if collection has exact length.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(has_len(3).check(&vec![1, 2, 3]));
/// assert!(!has_len(3).check(&vec![1, 2]));
/// ```
pub fn has_len(expected: usize) -> HasLen {
    HasLen { expected }
}

/// Predicate that checks minimum collection length.
#[derive(Clone, Copy, Debug)]
pub struct HasMinLen {
    min: usize,
}

impl<T> Predicate<Vec<T>> for HasMinLen {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.len() >= self.min
    }
}

impl<T> Predicate<[T]> for HasMinLen {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.len() >= self.min
    }
}

/// Create a predicate that checks if collection has at least min elements.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(has_min_len(2).check(&vec![1, 2, 3]));
/// assert!(has_min_len(2).check(&vec![1, 2]));
/// assert!(!has_min_len(2).check(&vec![1]));
/// ```
pub fn has_min_len(min: usize) -> HasMinLen {
    HasMinLen { min }
}

/// Predicate that checks maximum collection length.
#[derive(Clone, Copy, Debug)]
pub struct HasMaxLen {
    max: usize,
}

impl<T> Predicate<Vec<T>> for HasMaxLen {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.len() <= self.max
    }
}

impl<T> Predicate<[T]> for HasMaxLen {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.len() <= self.max
    }
}

/// Create a predicate that checks if collection has at most max elements.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(has_max_len(3).check(&vec![1, 2]));
/// assert!(has_max_len(3).check(&vec![1, 2, 3]));
/// assert!(!has_max_len(3).check(&vec![1, 2, 3, 4]));
/// ```
pub fn has_max_len(max: usize) -> HasMaxLen {
    HasMaxLen { max }
}

/// Predicate that checks if all elements satisfy a predicate.
#[derive(Clone, Copy, Debug)]
pub struct All<P>(pub P);

impl<T, P: Predicate<T>> Predicate<Vec<T>> for All<P> {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.iter().all(|item| self.0.check(item))
    }
}

impl<T, P: Predicate<T>> Predicate<[T]> for All<P> {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.iter().all(|item| self.0.check(item))
    }
}

/// Create a predicate that checks if all elements satisfy a condition.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(all(positive::<i32>()).check(&vec![1, 2, 3]));
/// assert!(!all(positive::<i32>()).check(&vec![1, -2, 3]));
/// ```
pub fn all<P>(predicate: P) -> All<P> {
    All(predicate)
}

/// Predicate that checks if any element satisfies a predicate.
#[derive(Clone, Copy, Debug)]
pub struct Any<P>(pub P);

impl<T, P: Predicate<T>> Predicate<Vec<T>> for Any<P> {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.iter().any(|item| self.0.check(item))
    }
}

impl<T, P: Predicate<T>> Predicate<[T]> for Any<P> {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.iter().any(|item| self.0.check(item))
    }
}

/// Create a predicate that checks if any element satisfies a condition.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(any(eq(5)).check(&vec![1, 5, 10]));
/// assert!(!any(eq(5)).check(&vec![1, 2, 3]));
/// ```
pub fn any<P>(predicate: P) -> Any<P> {
    Any(predicate)
}

/// Predicate that checks if collection contains a specific element.
#[derive(Clone, Copy, Debug)]
pub struct ContainsElement<T>(pub T);

impl<T: PartialEq + Send + Sync> Predicate<Vec<T>> for ContainsElement<T> {
    #[inline]
    fn check(&self, value: &Vec<T>) -> bool {
        value.contains(&self.0)
    }
}

impl<T: PartialEq + Send + Sync> Predicate<[T]> for ContainsElement<T> {
    #[inline]
    fn check(&self, value: &[T]) -> bool {
        value.contains(&self.0)
    }
}

/// Create a predicate that checks if collection contains a specific element.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(contains_element(5).check(&vec![1, 5, 10]));
/// assert!(!contains_element(5).check(&vec![1, 2, 3]));
/// ```
pub fn contains_element<T: PartialEq + Send + Sync>(element: T) -> ContainsElement<T> {
    ContainsElement(element)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{eq, positive};

    #[test]
    fn test_is_empty() {
        assert!(is_empty().check(&vec![] as &Vec<i32>));
        assert!(!is_empty().check(&vec![1]));
    }

    #[test]
    fn test_is_empty_slice() {
        let empty: &[i32] = &[];
        let non_empty: &[i32] = &[1, 2, 3];
        assert!(is_empty().check(empty));
        assert!(!is_empty().check(non_empty));
    }

    #[test]
    fn test_is_not_empty() {
        assert!(is_not_empty().check(&vec![1]));
        assert!(!is_not_empty().check(&vec![] as &Vec<i32>));
    }

    #[test]
    fn test_has_len() {
        assert!(has_len(3).check(&vec![1, 2, 3]));
        assert!(!has_len(3).check(&vec![1, 2]));
        assert!(!has_len(3).check(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_has_min_len() {
        assert!(has_min_len(2).check(&vec![1, 2, 3]));
        assert!(has_min_len(2).check(&vec![1, 2]));
        assert!(!has_min_len(2).check(&vec![1]));
    }

    #[test]
    fn test_has_max_len() {
        assert!(has_max_len(3).check(&vec![1, 2]));
        assert!(has_max_len(3).check(&vec![1, 2, 3]));
        assert!(!has_max_len(3).check(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_all() {
        assert!(all(positive::<i32>()).check(&vec![1, 2, 3]));
        assert!(!all(positive::<i32>()).check(&vec![1, -2, 3]));
        assert!(all(positive::<i32>()).check(&vec![] as &Vec<i32>)); // vacuously true
    }

    #[test]
    fn test_any() {
        assert!(any(eq(5)).check(&vec![1, 5, 10]));
        assert!(!any(eq(5)).check(&vec![1, 2, 3]));
        assert!(!any(eq(5)).check(&vec![] as &Vec<i32>)); // empty has no matches
    }

    #[test]
    fn test_contains_element() {
        assert!(contains_element(5).check(&vec![1, 5, 10]));
        assert!(!contains_element(5).check(&vec![1, 2, 3]));
    }

    #[test]
    fn test_contains_element_slice() {
        let v: &[i32] = &[1, 5, 10];
        assert!(contains_element(5).check(v));
        assert!(!contains_element(7).check(v));
    }
}
