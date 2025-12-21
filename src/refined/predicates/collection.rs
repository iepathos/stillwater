//! Collection predicates for refined types
//!
//! This module provides predicates for constraining collections:
//! - [`NonEmpty`] from string module also works for `Vec<T>`
//! - [`MaxSize<N>`]: Collection size <= N
//! - [`MinSize<N>`]: Collection size >= N
//!
//! # Example
//!
//! ```rust
//! use stillwater::refined::{Refined, NonEmpty, MaxSize, MinSize};
//!
//! // Non-empty vector
//! type NonEmptyVec<T> = Refined<Vec<T>, NonEmpty>;
//! let nums = NonEmptyVec::<i32>::new(vec![1, 2, 3]).unwrap();
//! assert!(NonEmptyVec::<i32>::new(vec![]).is_err());
//!
//! // Size constraints
//! type SmallList<T> = Refined<Vec<T>, MaxSize<5>>;
//! let small = SmallList::<i32>::new(vec![1, 2, 3]).unwrap();
//! ```

use super::super::Predicate;
use super::string::NonEmpty;

// NonEmpty also works for Vec<T>
impl<T: Send + Sync + 'static> Predicate<Vec<T>> for NonEmpty {
    type Error = &'static str;

    fn check(value: &Vec<T>) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err("collection cannot be empty")
        } else {
            Ok(())
        }
    }

    fn description() -> &'static str {
        "non-empty collection"
    }
}

/// Collection size must be at most N
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, MaxSize};
///
/// type SmallVec<T> = Refined<Vec<T>, MaxSize<3>>;
///
/// let v = SmallVec::<i32>::new(vec![1, 2, 3]).unwrap();
/// assert!(SmallVec::<i32>::new(vec![1, 2, 3, 4]).is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MaxSize<const N: usize>;

impl<const N: usize, T: Send + Sync + 'static> Predicate<Vec<T>> for MaxSize<N> {
    type Error = String;

    fn check(value: &Vec<T>) -> Result<(), Self::Error> {
        if value.len() <= N {
            Ok(())
        } else {
            Err(format!(
                "collection size {} exceeds maximum {}",
                value.len(),
                N
            ))
        }
    }

    fn description() -> &'static str {
        "collection with maximum size"
    }
}

/// Collection size must be at least N
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, MinSize};
///
/// type AtLeastTwo<T> = Refined<Vec<T>, MinSize<2>>;
///
/// let v = AtLeastTwo::<i32>::new(vec![1, 2]).unwrap();
/// assert!(AtLeastTwo::<i32>::new(vec![1]).is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MinSize<const N: usize>;

impl<const N: usize, T: Send + Sync + 'static> Predicate<Vec<T>> for MinSize<N> {
    type Error = String;

    fn check(value: &Vec<T>) -> Result<(), Self::Error> {
        if value.len() >= N {
            Ok(())
        } else {
            Err(format!(
                "collection size {} is less than minimum {}",
                value.len(),
                N
            ))
        }
    }

    fn description() -> &'static str {
        "collection with minimum size"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refined::Refined;

    type NonEmptyVec = Refined<Vec<i32>, NonEmpty>;
    type SmallVec = Refined<Vec<i32>, MaxSize<3>>;
    type AtLeastTwo = Refined<Vec<i32>, MinSize<2>>;

    #[test]
    fn test_non_empty_vec_success() {
        assert!(NonEmptyVec::new(vec![1]).is_ok());
        assert!(NonEmptyVec::new(vec![1, 2, 3]).is_ok());
    }

    #[test]
    fn test_non_empty_vec_failure() {
        let result = NonEmptyVec::new(vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "collection cannot be empty");
    }

    #[test]
    fn test_max_size_success() {
        assert!(SmallVec::new(vec![]).is_ok());
        assert!(SmallVec::new(vec![1]).is_ok());
        assert!(SmallVec::new(vec![1, 2, 3]).is_ok()); // exactly 3
    }

    #[test]
    fn test_max_size_failure() {
        let result = SmallVec::new(vec![1, 2, 3, 4]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_min_size_success() {
        assert!(AtLeastTwo::new(vec![1, 2]).is_ok()); // exactly 2
        assert!(AtLeastTwo::new(vec![1, 2, 3]).is_ok());
    }

    #[test]
    fn test_min_size_failure() {
        assert!(AtLeastTwo::new(vec![]).is_err());
        assert!(AtLeastTwo::new(vec![1]).is_err());
    }

    #[test]
    fn test_descriptions() {
        // Test NonEmpty for Vec (different description than for String)
        assert_eq!(
            <NonEmpty as Predicate<Vec<i32>>>::description(),
            "non-empty collection"
        );
        assert_eq!(
            <MaxSize<3> as Predicate<Vec<i32>>>::description(),
            "collection with maximum size"
        );
        assert_eq!(
            <MinSize<2> as Predicate<Vec<i32>>>::description(),
            "collection with minimum size"
        );
    }
}
