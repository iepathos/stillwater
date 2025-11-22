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
}
