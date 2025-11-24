//! Monoid trait for types with identity elements
//!
//! A `Monoid` extends `Semigroup` by adding an identity element. This enables more
//! powerful composition patterns including folding without an initial value and
//! parallel reduction.
//!
//! # Mathematical Properties
//!
//! For a type to be a valid Monoid, it must satisfy:
//! 1. **Associativity** (from Semigroup):
//!    ```text
//!    a.combine(b).combine(c) == a.combine(b.combine(c))
//!    ```
//! 2. **Right Identity**:
//!    ```text
//!    a.combine(M::empty()) == a
//!    ```
//! 3. **Left Identity**:
//!    ```text
//!    M::empty().combine(a) == a
//!    ```
//!
//! # Examples
//!
//! ```
//! use stillwater::{Monoid, Semigroup};
//!
//! // Vec is a monoid with empty vector as identity
//! let v1 = vec![1, 2, 3];
//! let empty: Vec<i32> = Monoid::empty();
//! assert_eq!(v1.clone().combine(empty.clone()), v1);
//! assert_eq!(empty.combine(v1.clone()), v1);
//!
//! // String is a monoid with empty string as identity
//! let s = "hello".to_string();
//! let empty: String = Monoid::empty();
//! assert_eq!(s.clone().combine(empty.clone()), s);
//! ```
//!
//! # Numeric Monoids
//!
//! Since Rust primitives can't implement traits and numbers have multiple valid monoid
//! instances (addition vs multiplication), we provide wrapper types:
//!
//! ```
//! use stillwater::monoid::{Sum, fold_all};
//!
//! let numbers = vec![Sum(1), Sum(2), Sum(3), Sum(4)];
//! let total = fold_all(numbers);
//! assert_eq!(total, Sum(10));
//! ```

use crate::Semigroup;
use std::ops::{Add, Mul};

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

// Vec monoid - empty vector is identity
impl<T> Monoid for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }
}

// String monoid - empty string is identity
impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }
}

// Option monoid - None is identity (lifts inner semigroup)
// Note: Semigroup for Option is implemented in semigroup.rs
impl<T: Semigroup> Monoid for Option<T> {
    fn empty() -> Self {
        None
    }
}

// Macro for generating tuple Monoid implementations
macro_rules! impl_monoid_tuple {
    ($($idx:tt $T:ident),+) => {
        impl<$($T: Monoid),+> Monoid for ($($T,)+) {
            fn empty() -> Self {
                ($($T::empty(),)+)
            }
        }
    };
}

// Generate implementations for tuples of size 2 through 12
impl_monoid_tuple!(0 T1, 1 T2);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10, 10 T11);
impl_monoid_tuple!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10, 10 T11, 11 T12);

// Monoid instances for collection types
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Monoid for HashMap - empty map is identity
impl<K, V> Monoid for HashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Semigroup + Clone,
{
    fn empty() -> Self {
        HashMap::new()
    }
}

/// Monoid for HashSet - empty set is identity
impl<T> Monoid for HashSet<T>
where
    T: Eq + Hash,
{
    fn empty() -> Self {
        HashSet::new()
    }
}

/// Monoid for BTreeMap - empty map is identity
impl<K, V> Monoid for BTreeMap<K, V>
where
    K: Ord + Clone,
    V: Semigroup + Clone,
{
    fn empty() -> Self {
        BTreeMap::new()
    }
}

/// Monoid for BTreeSet - empty set is identity
impl<T> Monoid for BTreeSet<T>
where
    T: Ord,
{
    fn empty() -> Self {
        BTreeSet::new()
    }
}

/// Monoid for numeric types under addition.
///
/// Identity: 0
///
/// # Example
///
/// ```
/// use stillwater::monoid::{Sum, fold_all};
/// use stillwater::Semigroup;
///
/// let nums = vec![Sum(1), Sum(2), Sum(3)];
/// let total = fold_all(nums);
/// assert_eq!(total, Sum(6));
///
/// // Using combine directly
/// let result = Sum(5).combine(Sum(10));
/// assert_eq!(result, Sum(15));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sum<T>(pub T);

impl<T: Add<Output = T>> Semigroup for Sum<T> {
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
///
/// # Example
///
/// ```
/// use stillwater::monoid::{Product, fold_all};
/// use stillwater::Semigroup;
///
/// let nums = vec![Product(2), Product(3), Product(4)];
/// let result = fold_all(nums);
/// assert_eq!(result, Product(24));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Product<T>(pub T);

/// Helper trait for types with a multiplicative identity (1)
pub trait One {
    /// Returns the multiplicative identity element for this type
    fn one() -> Self;
}

impl One for i8 {
    fn one() -> Self {
        1
    }
}
impl One for i16 {
    fn one() -> Self {
        1
    }
}
impl One for i32 {
    fn one() -> Self {
        1
    }
}
impl One for i64 {
    fn one() -> Self {
        1
    }
}
impl One for i128 {
    fn one() -> Self {
        1
    }
}
impl One for isize {
    fn one() -> Self {
        1
    }
}
impl One for u8 {
    fn one() -> Self {
        1
    }
}
impl One for u16 {
    fn one() -> Self {
        1
    }
}
impl One for u32 {
    fn one() -> Self {
        1
    }
}
impl One for u64 {
    fn one() -> Self {
        1
    }
}
impl One for u128 {
    fn one() -> Self {
        1
    }
}
impl One for usize {
    fn one() -> Self {
        1
    }
}
impl One for f32 {
    fn one() -> Self {
        1.0
    }
}
impl One for f64 {
    fn one() -> Self {
        1.0
    }
}

impl<T: Mul<Output = T>> Semigroup for Product<T> {
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
/// Note: Requires a bounded type since identity is the minimum bound.
/// For unbounded types, use `Option<Max<T>>`.
///
/// # Example
///
/// ```
/// use stillwater::monoid::Max;
/// use stillwater::Semigroup;
///
/// let m1 = Max(5);
/// let m2 = Max(10);
/// assert_eq!(m1.combine(m2), Max(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Max<T>(pub T);

impl<T: Ord> Semigroup for Max<T> {
    fn combine(self, other: Self) -> Self {
        Max(self.0.max(other.0))
    }
}

// Note: We can't implement Monoid for Max<T> without Bounded trait
// Users should use Option<Max<T>> for unbounded types

/// Monoid for ordered types under minimum.
///
/// Note: Requires a bounded type since identity is the maximum bound.
/// For unbounded types, use `Option<Min<T>>`.
///
/// # Example
///
/// ```
/// use stillwater::monoid::Min;
/// use stillwater::Semigroup;
///
/// let m1 = Min(5);
/// let m2 = Min(10);
/// assert_eq!(m1.combine(m2), Min(5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Min<T>(pub T);

impl<T: Ord> Semigroup for Min<T> {
    fn combine(self, other: Self) -> Self {
        Min(self.0.min(other.0))
    }
}

// Note: We can't implement Monoid for Min<T> without Bounded trait
// Users should use Option<Min<T>> for unbounded types

/// Fold an iterator using the Monoid instance, starting with `empty()`.
///
/// This is more convenient than `Iterator::fold` when working with monoids
/// because the identity element is provided by the type.
///
/// # Example
///
/// ```
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
///
/// # With Sum
///
/// ```
/// use stillwater::monoid::{Sum, fold_all};
///
/// let numbers = vec![Sum(1), Sum(2), Sum(3), Sum(4)];
/// let total = fold_all(numbers);
/// assert_eq!(total, Sum(10));
/// ```
pub fn fold_all<M, I>(iter: I) -> M
where
    M: Monoid,
    I: IntoIterator<Item = M>,
{
    iter.into_iter().fold(M::empty(), |acc, x| acc.combine(x))
}

/// Alias for `fold_all` - reduces an iterator to a single value using the Monoid.
///
/// # Example
///
/// ```
/// use stillwater::monoid::reduce;
///
/// let strings = vec!["Hello".to_string(), " ".to_string(), "World".to_string()];
/// let result: String = reduce(strings);
/// assert_eq!(result, "Hello World");
/// ```
pub fn reduce<M, I>(iter: I) -> M
where
    M: Monoid,
    I: IntoIterator<Item = M>,
{
    fold_all(iter)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Identity law tests

    #[test]
    fn test_vec_right_identity() {
        let v = vec![1, 2, 3];
        let empty: Vec<i32> = Monoid::empty();
        assert_eq!(v.clone().combine(empty), v);
    }

    #[test]
    fn test_vec_left_identity() {
        let v = vec![1, 2, 3];
        let empty: Vec<i32> = Monoid::empty();
        assert_eq!(empty.combine(v.clone()), v);
    }

    #[test]
    fn test_string_right_identity() {
        let s = "hello".to_string();
        let empty: String = Monoid::empty();
        assert_eq!(s.clone().combine(empty), s);
    }

    #[test]
    fn test_string_left_identity() {
        let s = "hello".to_string();
        let empty: String = Monoid::empty();
        assert_eq!(empty.combine(s.clone()), s);
    }

    #[test]
    fn test_option_right_identity() {
        let v = Some(vec![1, 2, 3]);
        let empty: Option<Vec<i32>> = Monoid::empty();
        assert_eq!(v.clone().combine(empty), v);
    }

    #[test]
    fn test_option_left_identity() {
        let v = Some(vec![1, 2, 3]);
        let empty: Option<Vec<i32>> = Monoid::empty();
        assert_eq!(empty.combine(v.clone()), v);
    }

    #[test]
    fn test_option_combine() {
        let v1 = Some(vec![1, 2]);
        let v2 = Some(vec![3, 4]);
        assert_eq!(v1.combine(v2), Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_option_none_some() {
        let v1: Option<Vec<i32>> = None;
        let v2 = Some(vec![1, 2]);
        assert_eq!(v1.combine(v2), Some(vec![1, 2]));
    }

    #[test]
    fn test_option_some_none() {
        let v1 = Some(vec![1, 2]);
        let v2: Option<Vec<i32>> = None;
        assert_eq!(v1.combine(v2), Some(vec![1, 2]));
    }

    #[test]
    fn test_tuple_identity() {
        let t = (vec![1], "hello".to_string());
        let empty: (Vec<i32>, String) = Monoid::empty();
        assert_eq!(t.clone().combine(empty.clone()), t);
        assert_eq!(empty.combine(t.clone()), t);
    }

    // fold_all tests

    #[test]
    fn test_fold_all_vec() {
        let vecs = vec![vec![1], vec![2, 3], vec![4]];
        let result = fold_all(vecs);
        assert_eq!(result, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_fold_all_empty() {
        let vecs: Vec<Vec<i32>> = vec![];
        let result = fold_all(vecs);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_fold_all_string() {
        let strings = vec!["Hello".to_string(), " ".to_string(), "World".to_string()];
        let result = fold_all(strings);
        assert_eq!(result, "Hello World");
    }

    // Sum monoid tests

    #[test]
    fn test_sum_combine() {
        let s1 = Sum(5);
        let s2 = Sum(10);
        assert_eq!(s1.combine(s2), Sum(15));
    }

    #[test]
    fn test_sum_identity() {
        let s = Sum(42);
        let empty: Sum<i32> = Monoid::empty();
        assert_eq!(s.combine(empty), Sum(42));
        assert_eq!(empty.combine(s), Sum(42));
    }

    #[test]
    fn test_sum_fold_all() {
        let nums = vec![Sum(1), Sum(2), Sum(3), Sum(4)];
        let result = fold_all(nums);
        assert_eq!(result, Sum(10));
    }

    // Product monoid tests

    #[test]
    fn test_product_combine() {
        let p1 = Product(5);
        let p2 = Product(10);
        assert_eq!(p1.combine(p2), Product(50));
    }

    #[test]
    fn test_product_identity() {
        let p = Product(42);
        let empty: Product<i32> = Monoid::empty();
        assert_eq!(p.combine(empty), Product(42));
        assert_eq!(empty.combine(p), Product(42));
    }

    #[test]
    fn test_product_fold_all() {
        let nums = vec![Product(2), Product(3), Product(4)];
        let result = fold_all(nums);
        assert_eq!(result, Product(24));
    }

    // Max/Min tests (without Monoid, just Semigroup)

    #[test]
    fn test_max_combine() {
        let m1 = Max(5);
        let m2 = Max(10);
        assert_eq!(m1.combine(m2), Max(10));
    }

    #[test]
    fn test_min_combine() {
        let m1 = Min(5);
        let m2 = Min(10);
        assert_eq!(m1.combine(m2), Min(5));
    }

    // reduce alias test

    #[test]
    fn test_reduce() {
        let vecs = vec![vec![1], vec![2], vec![3]];
        let result: Vec<i32> = reduce(vecs);
        assert_eq!(result, vec![1, 2, 3]);
    }

    // Associativity tests (inherited from Semigroup but worth verifying)

    #[test]
    fn test_sum_associativity() {
        let a = Sum(1);
        let b = Sum(2);
        let c = Sum(3);

        let left = a.combine(b).combine(c);
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    #[test]
    fn test_product_associativity() {
        let a = Product(2);
        let b = Product(3);
        let c = Product(4);

        let left = a.combine(b).combine(c);
        let right = a.combine(b.combine(c));

        assert_eq!(left, right);
    }

    // Property-based tests
    #[cfg(test)]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            // Vec monoid laws
            #[test]
            fn prop_vec_right_identity(v: Vec<i32>) {
                let empty: Vec<i32> = Monoid::empty();
                prop_assert_eq!(v.clone().combine(empty), v);
            }

            #[test]
            fn prop_vec_left_identity(v: Vec<i32>) {
                let empty: Vec<i32> = Monoid::empty();
                prop_assert_eq!(empty.combine(v.clone()), v);
            }

            #[test]
            fn prop_vec_associativity(a: Vec<i32>, b: Vec<i32>, c: Vec<i32>) {
                let left = a.clone().combine(b.clone()).combine(c.clone());
                let right = a.combine(b.combine(c));
                prop_assert_eq!(left, right);
            }

            // String monoid laws
            #[test]
            fn prop_string_right_identity(s: String) {
                let empty: String = Monoid::empty();
                prop_assert_eq!(s.clone().combine(empty), s);
            }

            #[test]
            fn prop_string_left_identity(s: String) {
                let empty: String = Monoid::empty();
                prop_assert_eq!(empty.combine(s.clone()), s);
            }

            #[test]
            fn prop_string_associativity(a: String, b: String, c: String) {
                let left = a.clone().combine(b.clone()).combine(c.clone());
                let right = a.combine(b.combine(c));
                prop_assert_eq!(left, right);
            }

            // Sum monoid laws (use smaller numbers to avoid overflow)
            #[test]
            fn prop_sum_right_identity(n in -1000i32..1000i32) {
                let s = Sum(n);
                let empty: Sum<i32> = Monoid::empty();
                prop_assert_eq!(s.combine(empty), Sum(n));
            }

            #[test]
            fn prop_sum_left_identity(n in -1000i32..1000i32) {
                let s = Sum(n);
                let empty: Sum<i32> = Monoid::empty();
                prop_assert_eq!(empty.combine(s), Sum(n));
            }

            #[test]
            fn prop_sum_associativity(a in -1000i32..1000i32, b in -1000i32..1000i32, c in -1000i32..1000i32) {
                let sa = Sum(a);
                let sb = Sum(b);
                let sc = Sum(c);
                let left = sa.combine(sb).combine(sc);
                let right = sa.combine(sb.combine(sc));
                prop_assert_eq!(left, right);
            }

            // Product monoid laws (use smaller numbers to avoid overflow)
            #[test]
            fn prop_product_right_identity(n in -100i32..100i32) {
                let p = Product(n);
                let empty: Product<i32> = Monoid::empty();
                prop_assert_eq!(p.combine(empty), Product(n));
            }

            #[test]
            fn prop_product_left_identity(n in -100i32..100i32) {
                let p = Product(n);
                let empty: Product<i32> = Monoid::empty();
                prop_assert_eq!(empty.combine(p), Product(n));
            }

            #[test]
            fn prop_product_associativity(a in -10i32..10i32, b in -10i32..10i32, c in -10i32..10i32) {
                let pa = Product(a);
                let pb = Product(b);
                let pc = Product(c);
                let left = pa.combine(pb).combine(pc);
                let right = pa.combine(pb.combine(pc));
                prop_assert_eq!(left, right);
            }

            // Option monoid laws
            #[test]
            fn prop_option_right_identity(v: Option<Vec<i32>>) {
                let empty: Option<Vec<i32>> = Monoid::empty();
                prop_assert_eq!(v.clone().combine(empty), v);
            }

            #[test]
            fn prop_option_left_identity(v: Option<Vec<i32>>) {
                let empty: Option<Vec<i32>> = Monoid::empty();
                prop_assert_eq!(empty.combine(v.clone()), v);
            }

            #[test]
            fn prop_option_associativity(
                a: Option<Vec<i32>>,
                b: Option<Vec<i32>>,
                c: Option<Vec<i32>>
            ) {
                let left = a.clone().combine(b.clone()).combine(c.clone());
                let right = a.combine(b.combine(c));
                prop_assert_eq!(left, right);
            }

            // fold_all properties
            #[test]
            fn prop_fold_all_vec(vecs: Vec<Vec<i32>>) {
                let result = fold_all(vecs.clone());
                let expected: Vec<i32> = vecs.into_iter().flatten().collect();
                prop_assert_eq!(result, expected);
            }

            #[test]
            fn prop_fold_all_string(strings: Vec<String>) {
                let result = fold_all(strings.clone());
                let expected: String = strings.join("");
                prop_assert_eq!(result, expected);
            }
        }
    }
}
