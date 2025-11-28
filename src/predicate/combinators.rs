//! Core predicate trait and logical combinators
//!
//! This module provides the foundational `Predicate` trait and logical
//! combinators for composing predicates.

/// A composable predicate over values of type T.
///
/// Predicates can be combined using logical operators:
/// - `and`: Both predicates must be true
/// - `or`: Either predicate must be true
/// - `not`: Inverts the predicate
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let is_valid_age = ge(0).and(le(150));
/// assert!(is_valid_age.check(&25));
/// assert!(!is_valid_age.check(&-5));
/// ```
pub trait Predicate<T: ?Sized>: Send + Sync {
    /// Check if the value satisfies this predicate.
    fn check(&self, value: &T) -> bool;
}

// Blanket impl for closures
impl<T: ?Sized, F> Predicate<T> for F
where
    F: Fn(&T) -> bool + Send + Sync,
{
    #[inline]
    fn check(&self, value: &T) -> bool {
        self(value)
    }
}

/// Extension trait for predicate combinators.
///
/// Provides method chaining for combining predicates with logical operators.
/// All methods return concrete types for zero-cost abstraction.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let p = gt(0).and(lt(100)).not();
/// assert!(p.check(&-5));  // not (> 0 and < 100)
/// assert!(!p.check(&50)); // 50 is in range, so not() inverts to false
/// ```
pub trait PredicateExt<T: ?Sized>: Predicate<T> + Sized {
    /// Combine with AND logic.
    ///
    /// Returns a predicate that is true only when both predicates are true.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::predicate::*;
    ///
    /// let p = gt(0).and(lt(100));
    /// assert!(p.check(&50));
    /// assert!(!p.check(&0));
    /// assert!(!p.check(&100));
    /// ```
    fn and<P: Predicate<T>>(self, other: P) -> And<Self, P> {
        And(self, other)
    }

    /// Combine with OR logic.
    ///
    /// Returns a predicate that is true when either predicate is true.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::predicate::*;
    ///
    /// let p = lt(0).or(gt(100));
    /// assert!(p.check(&-5));
    /// assert!(p.check(&150));
    /// assert!(!p.check(&50));
    /// ```
    fn or<P: Predicate<T>>(self, other: P) -> Or<Self, P> {
        Or(self, other)
    }

    /// Invert the predicate.
    ///
    /// Returns a predicate that is true when the original predicate is false.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::predicate::*;
    ///
    /// let p = positive::<i32>().not();
    /// assert!(p.check(&-5));
    /// assert!(p.check(&0));
    /// assert!(!p.check(&5));
    /// ```
    fn not(self) -> Not<Self> {
        Not(self)
    }
}

impl<T: ?Sized, P: Predicate<T>> PredicateExt<T> for P {}

/// AND combinator - both predicates must be true.
#[derive(Clone, Copy, Debug)]
pub struct And<P1, P2>(pub P1, pub P2);

impl<T: ?Sized, P1: Predicate<T>, P2: Predicate<T>> Predicate<T> for And<P1, P2> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        self.0.check(value) && self.1.check(value)
    }
}

// Send + Sync are auto-derived when P1 and P2 are Send + Sync

/// OR combinator - either predicate must be true.
#[derive(Clone, Copy, Debug)]
pub struct Or<P1, P2>(pub P1, pub P2);

impl<T: ?Sized, P1: Predicate<T>, P2: Predicate<T>> Predicate<T> for Or<P1, P2> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        self.0.check(value) || self.1.check(value)
    }
}

// Send + Sync are auto-derived when P1 and P2 are Send + Sync

/// NOT combinator - inverts the predicate.
#[derive(Clone, Copy, Debug)]
pub struct Not<P>(pub P);

impl<T: ?Sized, P: Predicate<T>> Predicate<T> for Not<P> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        !self.0.check(value)
    }
}

// Send + Sync are auto-derived when P is Send + Sync

/// Check if all predicates are satisfied (const generic, zero-allocation).
///
/// Uses a fixed-size array to avoid heap allocation.
/// Note: all_of requires homogeneous predicate types.
/// For mixed predicates, use .and() chaining instead.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// // all_of requires all predicates to be the same type
/// let greater_than_bounds = all_of([gt(0), gt(-10), gt(-100)]);
/// assert!(greater_than_bounds.check(&50));
/// assert!(!greater_than_bounds.check(&-50));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AllOf<P, const N: usize>(pub [P; N]);

impl<T: ?Sized, P: Predicate<T>, const N: usize> Predicate<T> for AllOf<P, N> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        self.0.iter().all(|p| p.check(value))
    }
}

/// Create a predicate that checks if all given predicates are satisfied.
///
/// This uses const generics for zero-allocation predicate arrays.
/// Note: all_of requires homogeneous predicate types.
/// For mixed predicates, use .and() chaining instead.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// // all_of requires all predicates to be the same type
/// let greater_than_bounds = all_of([gt(0), gt(-10), gt(-100)]);
/// assert!(greater_than_bounds.check(&50));
/// assert!(!greater_than_bounds.check(&-50));
/// ```
pub fn all_of<P, const N: usize>(predicates: [P; N]) -> AllOf<P, N> {
    AllOf(predicates)
}

/// Check if any predicate is satisfied (const generic, zero-allocation).
///
/// Uses a fixed-size array to avoid heap allocation.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let special_values = any_of([eq(1), eq(5), eq(10)]);
/// assert!(special_values.check(&5));
/// assert!(!special_values.check(&7));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AnyOf<P, const N: usize>(pub [P; N]);

impl<T: ?Sized, P: Predicate<T>, const N: usize> Predicate<T> for AnyOf<P, N> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        self.0.iter().any(|p| p.check(value))
    }
}

/// Create a predicate that checks if any given predicate is satisfied.
///
/// This uses const generics for zero-allocation predicate arrays.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let special_values = any_of([eq(1), eq(5), eq(10)]);
/// assert!(special_values.check(&5));
/// assert!(!special_values.check(&7));
/// ```
pub fn any_of<P, const N: usize>(predicates: [P; N]) -> AnyOf<P, N> {
    AnyOf(predicates)
}

/// Check if no predicates are satisfied (const generic, zero-allocation).
///
/// Equivalent to `not(any_of(...))`.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let no_special = none_of([eq(1), eq(5), eq(10)]);
/// assert!(no_special.check(&7));
/// assert!(!no_special.check(&5));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct NoneOf<P, const N: usize>(pub [P; N]);

impl<T: ?Sized, P: Predicate<T>, const N: usize> Predicate<T> for NoneOf<P, N> {
    #[inline]
    fn check(&self, value: &T) -> bool {
        !self.0.iter().any(|p| p.check(value))
    }
}

/// Create a predicate that checks if no given predicates are satisfied.
///
/// This is equivalent to `not(any_of(...))` but more direct.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let no_special = none_of([eq(1), eq(5), eq(10)]);
/// assert!(no_special.check(&7));
/// assert!(!no_special.check(&5));
/// ```
pub fn none_of<P, const N: usize>(predicates: [P; N]) -> NoneOf<P, N> {
    NoneOf(predicates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{eq, gt, lt, positive};

    #[test]
    fn test_and() {
        let p = gt(0).and(lt(10));
        assert!(p.check(&5));
        assert!(!p.check(&0));
        assert!(!p.check(&10));
    }

    #[test]
    fn test_or() {
        let p = lt(0).or(gt(100));
        assert!(p.check(&-5));
        assert!(p.check(&150));
        assert!(!p.check(&50));
    }

    #[test]
    fn test_not() {
        let p = positive::<i32>().not();
        assert!(p.check(&-5));
        assert!(p.check(&0));
        assert!(!p.check(&5));
    }

    #[test]
    fn test_all_of() {
        // all_of requires homogeneous predicate types
        // For mixed predicates, use .and() chaining instead
        let bounds = gt(0).and(lt(100));
        assert!(bounds.check(&50));
        assert!(!bounds.check(&0));
        assert!(!bounds.check(&100));

        // Homogeneous example with all_of
        let greater_than_bounds = all_of([gt(0), gt(-10), gt(-100)]);
        assert!(greater_than_bounds.check(&50));
        assert!(!greater_than_bounds.check(&-50));
    }

    #[test]
    fn test_any_of() {
        let p = any_of([eq(1), eq(5), eq(10)]);
        assert!(p.check(&1));
        assert!(p.check(&5));
        assert!(p.check(&10));
        assert!(!p.check(&2));
    }

    #[test]
    fn test_none_of() {
        let p = none_of([eq(1), eq(5), eq(10)]);
        assert!(!p.check(&1));
        assert!(!p.check(&5));
        assert!(p.check(&2));
        assert!(p.check(&7));
    }

    #[test]
    fn test_complex_chain() {
        // p1.and(p2).or(p3).not()
        let p = gt(0).and(lt(10)).or(gt(100)).not();
        // Original: (0 < x < 10) or (x > 100)
        // Negated: not((0 < x < 10) or (x > 100))
        // = x <= 0 or x >= 10) and x <= 100
        assert!(p.check(&0)); // x <= 0
        assert!(p.check(&50)); // 10 <= x <= 100
        assert!(!p.check(&5)); // 0 < 5 < 10, so original is true, negated is false
        assert!(!p.check(&150)); // > 100, original true, negated false
    }

    #[test]
    fn test_closure_as_predicate() {
        let is_even = |x: &i32| x % 2 == 0;
        assert!(is_even.check(&4));
        assert!(!is_even.check(&3));

        // Can be combined
        let even_and_positive = is_even.and(positive());
        assert!(even_and_positive.check(&4));
        assert!(!even_and_positive.check(&-4));
    }
}
