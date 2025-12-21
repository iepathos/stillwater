//! Non-empty vector type for type-safe collections
//!
//! This module provides the `NonEmptyVec<T>` type, which is a vector guaranteed to contain
//! at least one element. This provides type-level guarantees that prevent runtime errors
//! when operations assume a non-empty collection.
//!
//! # Examples
//!
//! ```
//! use stillwater::NonEmptyVec;
//!
//! let nev = NonEmptyVec::new(1, vec![2, 3, 4]);
//! assert_eq!(nev.head(), &1);
//! assert_eq!(nev.tail(), &[2, 3, 4]);
//! assert_eq!(nev.len(), 4);
//! ```
//!
//! # Use Cases
//!
//! - Validation errors: When a `Validation` fails, there's always at least one error
//! - Aggregations: Operations like `head()`, `max()`, `min()` require non-empty data
//! - Type safety: Prevent `None`/`panic!` in operations that need elements

use crate::Semigroup;

/// A non-empty vector guaranteed to contain at least one element.
///
/// This type provides type-level guarantees that operations like `head()`,
/// `max()`, and `min()` will always succeed without returning `Option`.
///
/// # Example
///
/// ```
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// # Panics
    ///
    /// Panics if the vector is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
    /// let nev = NonEmptyVec::from_vec_unchecked(vec![1, 2, 3]);
    /// assert_eq!(nev.len(), 3);
    /// ```
    ///
    /// ```should_panic
    /// use stillwater::NonEmptyVec;
    ///
    /// let nev = NonEmptyVec::from_vec_unchecked(Vec::<i32>::new()); // panics
    /// ```
    pub fn from_vec_unchecked(vec: Vec<T>) -> Self {
        Self::from_vec(vec).expect("NonEmptyVec::from_vec_unchecked called on empty Vec")
    }

    /// Get the first element (always succeeds).
    ///
    /// # Example
    ///
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// assert_eq!(nev.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    /// Check if the vector is empty.
    ///
    /// Always returns `false` since a NonEmptyVec is guaranteed to have at least one element.
    ///
    /// This method exists to satisfy clippy's `len_without_is_empty` lint.
    ///
    /// # Example
    ///
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
    /// let nev = NonEmptyVec::singleton(42);
    /// assert!(!nev.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Push an element to the end.
    ///
    /// # Example
    ///
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
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
    /// ```
    /// use stillwater::NonEmptyVec;
    ///
    /// let nev = NonEmptyVec::new(1, vec![2, 3]);
    /// let sum: i32 = nev.iter().sum();
    /// assert_eq!(sum, 6);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        std::iter::once(&self.head).chain(self.tail.iter())
    }
}

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
        std::iter::once(self.head).chain(self.tail)
    }
}

// Note: We cannot implement FromIterator for Option<NonEmptyVec<T>> due to orphan rules.
// Instead, use NonEmptyVec::from_vec(vec) where vec is collected from an iterator.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton() {
        let nev = NonEmptyVec::singleton(42);
        assert_eq!(nev.head(), &42);
        assert_eq!(nev.tail(), &[] as &[i32]);
        assert_eq!(nev.len(), 1);
    }

    #[test]
    fn test_new() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        assert_eq!(nev.head(), &1);
        assert_eq!(nev.tail(), &[2, 3]);
        assert_eq!(nev.len(), 3);
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
    fn test_from_vec_unchecked() {
        let nev = NonEmptyVec::from_vec_unchecked(vec![1, 2, 3]);
        assert_eq!(nev.head(), &1);
        assert_eq!(nev.tail(), &[2, 3]);
    }

    #[test]
    #[should_panic(expected = "NonEmptyVec::from_vec_unchecked called on empty Vec")]
    fn test_from_vec_unchecked_panics() {
        NonEmptyVec::from_vec_unchecked(Vec::<i32>::new());
    }

    #[test]
    fn test_last() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        assert_eq!(nev.last(), &3);

        let single = NonEmptyVec::singleton(42);
        assert_eq!(single.last(), &42);
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

        let nev2 = NonEmptyVec::singleton(1);
        let empty = nev2.filter(|x| x % 2 == 0);
        assert_eq!(empty, Vec::<i32>::new());
    }

    #[test]
    fn test_into_vec() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        let vec = nev.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_iter() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        let sum: i32 = nev.iter().sum();
        assert_eq!(sum, 6);

        let collected: Vec<_> = nev.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
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

    #[test]
    fn test_from_vec_with_iterator() {
        // Since we can't implement FromIterator, test the pattern of collect + from_vec
        let vec: Vec<i32> = vec![1, 2, 3].into_iter().collect();
        let nev = NonEmptyVec::from_vec(vec);
        assert!(nev.is_some());
        assert_eq!(nev.unwrap().into_vec(), vec![1, 2, 3]);

        let vec_empty: Vec<i32> = vec![].into_iter().collect();
        let nev_empty = NonEmptyVec::from_vec(vec_empty);
        assert!(nev_empty.is_none());
    }

    #[test]
    fn test_index() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        assert_eq!(nev[0], 1);
        assert_eq!(nev[1], 2);
        assert_eq!(nev[2], 3);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let nev = NonEmptyVec::singleton(42);
        let _ = nev[1]; // Should panic
    }
}
