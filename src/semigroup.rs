//! Semigroup trait for associative operations
//!
//! A Semigroup is a type with an associative binary operation. This trait is fundamental
//! for error accumulation in validation scenarios, where we want to combine multiple errors
//! together rather than short-circuiting on the first error.
//!
//! # Mathematical Properties
//!
//! For a type to be a valid Semigroup, the `combine` operation must be associative:
//! ```text
//! a.combine(b).combine(c) == a.combine(b.combine(c))
//! ```
//!
//! # Examples
//!
//! ```
//! use stillwater::Semigroup;
//!
//! // Combining vectors
//! let v1 = vec![1, 2, 3];
//! let v2 = vec![4, 5, 6];
//! assert_eq!(v1.combine(v2), vec![1, 2, 3, 4, 5, 6]);
//!
//! // Combining strings
//! let s1 = "Hello, ".to_string();
//! let s2 = "World!".to_string();
//! assert_eq!(s1.combine(s2), "Hello, World!");
//!
//! // Combining tuples component-wise
//! let t1 = (vec![1], "a".to_string());
//! let t2 = (vec![2], "b".to_string());
//! assert_eq!(t1.combine(t2), (vec![1, 2], "ab".to_string()));
//! ```
//!
//! # Custom Implementations
//!
//! You can implement Semigroup for your own types:
//!
//! ```
//! use stillwater::Semigroup;
//!
//! #[derive(Debug, PartialEq)]
//! struct ValidationErrors(Vec<String>);
//!
//! impl Semigroup for ValidationErrors {
//!     fn combine(mut self, other: Self) -> Self {
//!         self.0.extend(other.0);
//!         self
//!     }
//! }
//! ```

/// A type that supports an associative binary operation
///
/// # Laws
///
/// Implementations must satisfy the associativity law:
/// ```text
/// a.combine(b).combine(c) == a.combine(b.combine(c))
/// ```
///
/// # Note on Ownership
///
/// The `combine` method takes `self` by value, not by reference. If you need to
/// preserve the original values, you must clone them before combining.
pub trait Semigroup: Sized {
    /// Combine this value with another value associatively
    ///
    /// # Examples
    ///
    /// ```
    /// use stillwater::Semigroup;
    ///
    /// let v1 = vec![1, 2];
    /// let v2 = vec![3, 4];
    /// let result = v1.combine(v2);
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    fn combine(self, other: Self) -> Self;
}

// Implementation for Vec<T>
impl<T> Semigroup for Vec<T> {
    #[inline]
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

// Implementation for String
impl Semigroup for String {
    #[inline]
    fn combine(mut self, other: Self) -> Self {
        self.push_str(&other);
        self
    }
}

// Macro for generating tuple implementations
macro_rules! impl_semigroup_tuple {
    ($($idx:tt $T:ident),+) => {
        impl<$($T: Semigroup),+> Semigroup for ($($T,)+) {
            #[inline]
            fn combine(self, other: Self) -> Self {
                (
                    $(self.$idx.combine(other.$idx)),+
                )
            }
        }
    };
}

// Generate implementations for tuples of size 2 through 12
impl_semigroup_tuple!(0 T1, 1 T2);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10, 10 T11);
impl_semigroup_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10, 10 T11, 11 T12);

// Implementation for HashMap<K, V>
use std::collections::HashMap;
use std::hash::Hash;

/// Semigroup for HashMap that merges maps, combining values with the same key.
///
/// When keys conflict, values are combined using their Semigroup instance.
///
/// # Example
///
/// ```
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
    K: Eq + Hash + Clone,
    V: Semigroup + Clone,
{
    fn combine(mut self, other: Self) -> Self {
        for (key, value) in other {
            self.entry(key.clone())
                .and_modify(|existing| {
                    *existing = existing.clone().combine(value.clone());
                })
                .or_insert(value);
        }
        self
    }
}

// Implementation for HashSet<T>
use std::collections::HashSet;

/// Semigroup for HashSet using union.
///
/// # Example
///
/// ```
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

// Implementation for BTreeMap<K, V>
use std::collections::BTreeMap;

/// Semigroup for BTreeMap that merges maps, combining values with the same key.
///
/// Same semantics as HashMap but maintains ordered keys.
///
/// # Example
///
/// ```
/// use std::collections::BTreeMap;
/// use stillwater::Semigroup;
///
/// let mut map1 = BTreeMap::new();
/// map1.insert("a", vec![1, 2]);
///
/// let mut map2 = BTreeMap::new();
/// map2.insert("a", vec![3, 4]);
/// map2.insert("b", vec![5]);
///
/// let combined = map1.combine(map2);
/// assert_eq!(combined.get("a"), Some(&vec![1, 2, 3, 4]));
/// assert_eq!(combined.get("b"), Some(&vec![5]));
/// ```
impl<K, V> Semigroup for BTreeMap<K, V>
where
    K: Ord + Clone,
    V: Semigroup + Clone,
{
    fn combine(mut self, other: Self) -> Self {
        for (key, value) in other {
            self.entry(key.clone())
                .and_modify(|existing| {
                    *existing = existing.clone().combine(value.clone());
                })
                .or_insert(value);
        }
        self
    }
}

// Implementation for BTreeSet<T>
use std::collections::BTreeSet;

/// Semigroup for BTreeSet using union.
///
/// Same semantics as HashSet but maintains ordered elements.
///
/// # Example
///
/// ```
/// use std::collections::BTreeSet;
/// use stillwater::Semigroup;
///
/// let set1: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
/// let set2: BTreeSet<_> = [3, 4, 5].iter().cloned().collect();
///
/// let combined = set1.combine(set2);
/// assert_eq!(combined.len(), 5); // {1, 2, 3, 4, 5}
/// ```
impl<T> Semigroup for BTreeSet<T>
where
    T: Ord,
{
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

// Implementation for Option<T>
/// Semigroup for Option that lifts the inner Semigroup.
///
/// - `Some(a).combine(Some(b))` = `Some(a.combine(b))`
/// - `Some(a).combine(None)` = `Some(a)`
/// - `None.combine(Some(b))` = `Some(b)`
/// - `None.combine(None)` = `None`
///
/// # Example
///
/// ```
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

// Wrapper types for alternative semantics

/// Wrapper type that keeps the first value (left-biased).
///
/// # Example
///
/// ```
/// use stillwater::{First, Semigroup};
///
/// let first = First(1).combine(First(2));
/// assert_eq!(first.0, 1); // Keeps first
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
/// ```
/// use stillwater::{Last, Semigroup};
///
/// let last = Last(1).combine(Last(2));
/// assert_eq!(last.0, 2); // Keeps last
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
/// ```
/// use std::collections::HashSet;
/// use stillwater::{Intersection, Semigroup};
///
/// let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
/// let set2: HashSet<_> = [2, 3, 4].iter().cloned().collect();
///
/// let i1 = Intersection(set1);
/// let i2 = Intersection(set2);
/// let result = i1.combine(i2);
///
/// let expected: HashSet<_> = [2, 3].iter().cloned().collect();
/// assert_eq!(result.0, expected); // Intersection
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

impl<T> Semigroup for Intersection<BTreeSet<T>>
where
    T: Ord + Clone,
{
    fn combine(self, other: Self) -> Self {
        Intersection(self.0.intersection(&other.0).cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests
    #[test]
    fn test_vec_semigroup() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5, 6];
        assert_eq!(v1.combine(v2), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_vec_semigroup_empty() {
        let v1: Vec<i32> = vec![];
        let v2 = vec![1, 2, 3];
        assert_eq!(v1.combine(v2), vec![1, 2, 3]);
    }

    #[test]
    fn test_string_semigroup() {
        let s1 = "Hello, ".to_string();
        let s2 = "World!".to_string();
        assert_eq!(s1.combine(s2), "Hello, World!");
    }

    #[test]
    fn test_string_semigroup_empty() {
        let s1 = "".to_string();
        let s2 = "Hello".to_string();
        assert_eq!(s1.combine(s2), "Hello");
    }

    #[test]
    fn test_tuple_2_semigroup() {
        let t1 = (vec![1], "a".to_string());
        let t2 = (vec![2], "b".to_string());
        assert_eq!(t1.combine(t2), (vec![1, 2], "ab".to_string()));
    }

    #[test]
    fn test_tuple_3_semigroup() {
        let t1 = (vec![1], "a".to_string(), vec!["x".to_string()]);
        let t2 = (vec![2], "b".to_string(), vec!["y".to_string()]);
        assert_eq!(
            t1.combine(t2),
            (
                vec![1, 2],
                "ab".to_string(),
                vec!["x".to_string(), "y".to_string()]
            )
        );
    }

    // Associativity tests
    #[test]
    fn test_vec_associativity() {
        let a = vec![1, 2];
        let b = vec![3, 4];
        let c = vec![5, 6];

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_string_associativity() {
        let a = "hello".to_string();
        let b = " ".to_string();
        let c = "world".to_string();

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_tuple_associativity() {
        let a = (vec![1], "a".to_string());
        let b = (vec![2], "b".to_string());
        let c = (vec![3], "c".to_string());

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Property-based tests would go here if we had proptest/quickcheck
    // For now, we'll test with multiple concrete examples

    #[test]
    fn test_vec_multiple_combines() {
        let result = vec![1].combine(vec![2]).combine(vec![3]).combine(vec![4]);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_string_multiple_combines() {
        let result = "a"
            .to_string()
            .combine("b".to_string())
            .combine("c".to_string())
            .combine("d".to_string());
        assert_eq!(result, "abcd");
    }

    // Test larger tuples
    #[test]
    fn test_large_tuple() {
        let t1 = (vec![1], "a".to_string(), vec![2], "b".to_string(), vec![3]);
        let t2 = (vec![4], "c".to_string(), vec![5], "d".to_string(), vec![6]);
        assert_eq!(
            t1.combine(t2),
            (
                vec![1, 4],
                "ac".to_string(),
                vec![2, 5],
                "bd".to_string(),
                vec![3, 6],
            )
        );
    }

    // Tests for HashMap
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
    fn test_hashmap_no_overlap() {
        let mut map1 = HashMap::new();
        map1.insert("a", vec![1, 2]);

        let mut map2 = HashMap::new();
        map2.insert("b", vec![3, 4]);

        let result = map1.combine(map2);
        assert_eq!(result.get("a"), Some(&vec![1, 2]));
        assert_eq!(result.get("b"), Some(&vec![3, 4]));
    }

    #[test]
    fn test_hashmap_associativity() {
        let mut a = HashMap::new();
        a.insert("x", vec![1]);

        let mut b = HashMap::new();
        b.insert("x", vec![2]);

        let mut c = HashMap::new();
        c.insert("x", vec![3]);

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Tests for HashSet
    #[test]
    fn test_hashset_union() {
        let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
        let set2: HashSet<_> = [3, 4, 5].iter().cloned().collect();

        let result = set1.combine(set2);
        assert_eq!(result.len(), 5);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(result.contains(&3));
        assert!(result.contains(&4));
        assert!(result.contains(&5));
    }

    #[test]
    fn test_hashset_associativity() {
        let a: HashSet<_> = [1, 2].iter().cloned().collect();
        let b: HashSet<_> = [2, 3].iter().cloned().collect();
        let c: HashSet<_> = [3, 4].iter().cloned().collect();

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Tests for BTreeMap
    #[test]
    fn test_btreemap_combine() {
        let mut map1 = BTreeMap::new();
        map1.insert("a", vec![1, 2]);

        let mut map2 = BTreeMap::new();
        map2.insert("a", vec![3, 4]);
        map2.insert("b", vec![5]);

        let result = map1.combine(map2);
        assert_eq!(result.get("a"), Some(&vec![1, 2, 3, 4]));
        assert_eq!(result.get("b"), Some(&vec![5]));
    }

    #[test]
    fn test_btreemap_associativity() {
        let mut a = BTreeMap::new();
        a.insert("x", vec![1]);

        let mut b = BTreeMap::new();
        b.insert("x", vec![2]);

        let mut c = BTreeMap::new();
        c.insert("x", vec![3]);

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Tests for BTreeSet
    #[test]
    fn test_btreeset_union() {
        let set1: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
        let set2: BTreeSet<_> = [3, 4, 5].iter().cloned().collect();

        let result = set1.combine(set2);
        assert_eq!(result.len(), 5);
        assert!(result.contains(&1));
        assert!(result.contains(&5));
    }

    #[test]
    fn test_btreeset_associativity() {
        let a: BTreeSet<_> = [1, 2].iter().cloned().collect();
        let b: BTreeSet<_> = [2, 3].iter().cloned().collect();
        let c: BTreeSet<_> = [3, 4].iter().cloned().collect();

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Tests for Option
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
    fn test_option_both_none() {
        let none1: Option<Vec<i32>> = None;
        let none2: Option<Vec<i32>> = None;
        assert_eq!(none1.combine(none2), None);
    }

    #[test]
    fn test_option_associativity() {
        let a = Some(vec![1]);
        let b = Some(vec![2]);
        let c = Some(vec![3]);

        let left = a.clone().combine(b.clone()).combine(c.clone());
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Tests for First and Last
    #[test]
    fn test_first_last() {
        assert_eq!(First(1).combine(First(2)), First(1));
        assert_eq!(Last(1).combine(Last(2)), Last(2));
    }

    #[test]
    fn test_first_associativity() {
        let a = First(1);
        let b = First(2);
        let c = First(3);

        let left = a.combine(b).combine(c);
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_last_associativity() {
        let a = Last(1);
        let b = Last(2);
        let c = Last(3);

        let left = a.combine(b).combine(c);
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Tests for Intersection
    #[test]
    fn test_intersection_hashset() {
        let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
        let set2: HashSet<_> = [2, 3, 4].iter().cloned().collect();

        let i1 = Intersection(set1);
        let i2 = Intersection(set2);
        let result = i1.combine(i2);

        let expected: HashSet<_> = [2, 3].iter().cloned().collect();
        assert_eq!(result.0, expected);
    }

    #[test]
    fn test_intersection_btreeset() {
        let set1: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
        let set2: BTreeSet<_> = [2, 3, 4].iter().cloned().collect();

        let i1 = Intersection(set1);
        let i2 = Intersection(set2);
        let result = i1.combine(i2);

        let expected: BTreeSet<_> = [2, 3].iter().cloned().collect();
        assert_eq!(result.0, expected);
    }

    #[test]
    fn test_intersection_associativity() {
        let a: HashSet<_> = [1, 2, 3, 4].iter().cloned().collect();
        let b: HashSet<_> = [2, 3, 4, 5].iter().cloned().collect();
        let c: HashSet<_> = [3, 4, 5, 6].iter().cloned().collect();

        let left = Intersection(a.clone())
            .combine(Intersection(b.clone()))
            .combine(Intersection(c.clone()));
        let right = Intersection(a).combine(Intersection(b).combine(Intersection(c)));

        assert_eq!(left.0, right.0);
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
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

            #[test]
            fn prop_btreemap_associative(
                a: BTreeMap<String, Vec<i32>>,
                b: BTreeMap<String, Vec<i32>>,
                c: BTreeMap<String, Vec<i32>>,
            ) {
                let left = a.clone().combine(b.clone()).combine(c.clone());
                let right = a.combine(b.combine(c));
                prop_assert_eq!(left, right);
            }

            #[test]
            fn prop_btreeset_associative(
                a: BTreeSet<i32>,
                b: BTreeSet<i32>,
                c: BTreeSet<i32>,
            ) {
                let left = a.clone().combine(b.clone()).combine(c.clone());
                let right = a.combine(b.combine(c));
                prop_assert_eq!(left, right);
            }

            #[test]
            fn prop_option_associative(
                a: Option<Vec<i32>>,
                b: Option<Vec<i32>>,
                c: Option<Vec<i32>>,
            ) {
                let left = a.clone().combine(b.clone()).combine(c.clone());
                let right = a.combine(b.combine(c));
                prop_assert_eq!(left, right);
            }

            #[test]
            fn prop_first_associative(a: i32, b: i32, c: i32) {
                let fa = First(a);
                let fb = First(b);
                let fc = First(c);
                let left = fa.combine(fb).combine(fc);
                let right = fa.combine(fb.combine(fc));
                prop_assert_eq!(left, right);
            }

            #[test]
            fn prop_last_associative(a: i32, b: i32, c: i32) {
                let la = Last(a);
                let lb = Last(b);
                let lc = Last(c);
                let left = la.combine(lb).combine(lc);
                let right = la.combine(lb.combine(lc));
                prop_assert_eq!(left, right);
            }

            #[test]
            fn prop_intersection_associative(
                a: HashSet<i32>,
                b: HashSet<i32>,
                c: HashSet<i32>,
            ) {
                let left = Intersection(a.clone())
                    .combine(Intersection(b.clone()))
                    .combine(Intersection(c.clone()));
                let right = Intersection(a).combine(
                    Intersection(b).combine(Intersection(c))
                );
                prop_assert_eq!(left.0, right.0);
            }
        }
    }
}
