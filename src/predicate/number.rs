//! Number predicates
//!
//! This module provides common predicates for numeric validation.

use super::combinators::Predicate;
use std::cmp::PartialOrd;

/// Predicate for equality.
#[derive(Clone, Copy, Debug)]
pub struct Eq<T>(pub T);

impl<T: PartialEq + Send + Sync> Predicate<T> for Eq<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value == self.0
    }
}

/// Create a predicate that checks for equality.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(eq(5).check(&5));
/// assert!(!eq(5).check(&4));
/// ```
pub fn eq<T: PartialEq + Send + Sync>(value: T) -> Eq<T> {
    Eq(value)
}

/// Predicate for not equal.
#[derive(Clone, Copy, Debug)]
pub struct Ne<T>(pub T);

impl<T: PartialEq + Send + Sync> Predicate<T> for Ne<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value != self.0
    }
}

/// Create a predicate that checks for inequality.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(ne(5).check(&4));
/// assert!(ne(5).check(&6));
/// assert!(!ne(5).check(&5));
/// ```
pub fn ne<T: PartialEq + Send + Sync>(value: T) -> Ne<T> {
    Ne(value)
}

/// Predicate for greater than.
#[derive(Clone, Copy, Debug)]
pub struct Gt<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Gt<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value > self.0
    }
}

/// Create a predicate that checks if value is greater than threshold.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(gt(5).check(&6));
/// assert!(!gt(5).check(&5));
/// assert!(!gt(5).check(&4));
/// ```
pub fn gt<T: PartialOrd + Send + Sync>(value: T) -> Gt<T> {
    Gt(value)
}

/// Predicate for greater than or equal.
#[derive(Clone, Copy, Debug)]
pub struct Ge<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Ge<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value >= self.0
    }
}

/// Create a predicate that checks if value is greater than or equal to threshold.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(ge(5).check(&6));
/// assert!(ge(5).check(&5));
/// assert!(!ge(5).check(&4));
/// ```
pub fn ge<T: PartialOrd + Send + Sync>(value: T) -> Ge<T> {
    Ge(value)
}

/// Predicate for less than.
#[derive(Clone, Copy, Debug)]
pub struct Lt<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Lt<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value < self.0
    }
}

/// Create a predicate that checks if value is less than threshold.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(lt(5).check(&4));
/// assert!(!lt(5).check(&5));
/// assert!(!lt(5).check(&6));
/// ```
pub fn lt<T: PartialOrd + Send + Sync>(value: T) -> Lt<T> {
    Lt(value)
}

/// Predicate for less than or equal.
#[derive(Clone, Copy, Debug)]
pub struct Le<T>(pub T);

impl<T: PartialOrd + Send + Sync> Predicate<T> for Le<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value <= self.0
    }
}

/// Create a predicate that checks if value is less than or equal to threshold.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(le(5).check(&4));
/// assert!(le(5).check(&5));
/// assert!(!le(5).check(&6));
/// ```
pub fn le<T: PartialOrd + Send + Sync>(value: T) -> Le<T> {
    Le(value)
}

/// Predicate for value in range (inclusive).
#[derive(Clone, Copy, Debug)]
pub struct Between<T> {
    min: T,
    max: T,
}

impl<T: PartialOrd + Send + Sync> Predicate<T> for Between<T> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        *value >= self.min && *value <= self.max
    }
}

/// Create a predicate that checks if value is between min and max (inclusive).
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let p = between(0, 100);
/// assert!(p.check(&0));
/// assert!(p.check(&50));
/// assert!(p.check(&100));
/// assert!(!p.check(&-1));
/// assert!(!p.check(&101));
/// ```
pub fn between<T: PartialOrd + Send + Sync>(min: T, max: T) -> Between<T> {
    Between { min, max }
}

/// Create a predicate that checks if value is positive (greater than zero).
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let p = positive::<i32>();
/// assert!(p.check(&1));
/// assert!(!p.check(&0));
/// assert!(!p.check(&-1));
/// ```
pub fn positive<T>() -> Gt<T>
where
    T: PartialOrd + Default + Send + Sync,
{
    Gt(T::default())
}

/// Create a predicate that checks if value is negative (less than zero).
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let p = negative::<i32>();
/// assert!(p.check(&-1));
/// assert!(!p.check(&0));
/// assert!(!p.check(&1));
/// ```
pub fn negative<T>() -> Lt<T>
where
    T: PartialOrd + Default + Send + Sync,
{
    Lt(T::default())
}

/// Create a predicate that checks if value is non-negative (greater than or equal to zero).
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let p = non_negative::<i32>();
/// assert!(p.check(&0));
/// assert!(p.check(&1));
/// assert!(!p.check(&-1));
/// ```
pub fn non_negative<T>() -> Ge<T>
where
    T: PartialOrd + Default + Send + Sync,
{
    Ge(T::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::PredicateExt;

    #[test]
    fn test_eq() {
        assert!(eq(5).check(&5));
        assert!(!eq(5).check(&4));
    }

    #[test]
    fn test_ne() {
        let p = ne(5);
        assert!(p.check(&4));
        assert!(p.check(&6));
        assert!(!p.check(&5));
    }

    #[test]
    fn test_gt() {
        assert!(gt(5).check(&6));
        assert!(!gt(5).check(&5));
        assert!(!gt(5).check(&4));
    }

    #[test]
    fn test_ge() {
        assert!(ge(5).check(&6));
        assert!(ge(5).check(&5));
        assert!(!ge(5).check(&4));
    }

    #[test]
    fn test_lt() {
        assert!(lt(5).check(&4));
        assert!(!lt(5).check(&5));
        assert!(!lt(5).check(&6));
    }

    #[test]
    fn test_le() {
        assert!(le(5).check(&4));
        assert!(le(5).check(&5));
        assert!(!le(5).check(&6));
    }

    #[test]
    fn test_between() {
        let p = between(0, 100);
        assert!(p.check(&0));
        assert!(p.check(&50));
        assert!(p.check(&100));
        assert!(!p.check(&-1));
        assert!(!p.check(&101));
    }

    #[test]
    fn test_positive() {
        let p = positive::<i32>();
        assert!(p.check(&1));
        assert!(!p.check(&0));
        assert!(!p.check(&-1));
    }

    #[test]
    fn test_negative() {
        let p = negative::<i32>();
        assert!(p.check(&-1));
        assert!(!p.check(&0));
        assert!(!p.check(&1));
    }

    #[test]
    fn test_non_negative() {
        let p = non_negative::<i32>();
        assert!(p.check(&0));
        assert!(p.check(&1));
        assert!(!p.check(&-1));
    }

    #[test]
    fn test_combined_number_predicates() {
        let p = gt(10).and(lt(20));
        assert!(p.check(&15));
        assert!(!p.check(&10));
        assert!(!p.check(&20));
        assert!(!p.check(&5));
        assert!(!p.check(&25));
    }

    #[test]
    fn test_with_floats() {
        let p = between(0.0_f64, 1.0_f64);
        assert!(p.check(&0.5));
        assert!(p.check(&0.0));
        assert!(p.check(&1.0));
        assert!(!p.check(&-0.1));
        assert!(!p.check(&1.1));
    }
}
