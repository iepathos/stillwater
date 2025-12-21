//! Predicate combinators for composing refinement predicates
//!
//! This module provides combinators for building complex predicates
//! from simpler ones:
//! - [`And<A, B>`]: Both predicates must hold
//! - [`Or<A, B>`]: At least one predicate must hold
//! - [`Not<A>`]: Predicate must NOT hold
//!
//! # Example
//!
//! ```rust
//! use stillwater::refined::{Refined, NonEmpty, Trimmed, And, MaxLength};
//!
//! // Combined predicates
//! type ValidUsername = Refined<String, And<And<NonEmpty, Trimmed>, MaxLength<20>>>;
//!
//! let user = ValidUsername::new("alice".to_string()).unwrap();
//! assert!(ValidUsername::new("".to_string()).is_err());
//! assert!(ValidUsername::new("  alice  ".to_string()).is_err());
//! ```

use std::fmt;
use std::marker::PhantomData;

use super::Predicate;

/// Both predicates must hold
///
/// The `And` combinator checks both predicates and collects all errors
/// if both fail.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, NonEmpty, Trimmed, And};
///
/// // String must be both non-empty AND trimmed
/// type CleanString = Refined<String, And<NonEmpty, Trimmed>>;
///
/// let s = CleanString::new("hello".to_string()).unwrap();
/// assert!(CleanString::new("".to_string()).is_err());
/// assert!(CleanString::new("  hello  ".to_string()).is_err());
/// ```
#[derive(Clone, Copy, Default)]
pub struct And<A, B>(PhantomData<(A, B)>);

impl<A, B> fmt::Debug for And<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "And<{}, {}>",
            std::any::type_name::<A>(),
            std::any::type_name::<B>()
        )
    }
}

impl<T, A, B> Predicate<T> for And<A, B>
where
    A: Predicate<T>,
    B: Predicate<T>,
{
    type Error = AndError<A::Error, B::Error>;

    fn check(value: &T) -> Result<(), Self::Error> {
        let a_result = A::check(value);
        let b_result = B::check(value);

        match (a_result, b_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(a), Ok(())) => Err(AndError::First(a)),
            (Ok(()), Err(b)) => Err(AndError::Second(b)),
            (Err(a), Err(b)) => Err(AndError::Both(a, b)),
        }
    }

    fn description() -> &'static str {
        "both predicates must hold"
    }
}

/// Error type for And combinator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndError<A, B> {
    /// First predicate failed
    First(A),
    /// Second predicate failed
    Second(B),
    /// Both predicates failed
    Both(A, B),
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for AndError<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AndError::First(a) => write!(f, "{}", a),
            AndError::Second(b) => write!(f, "{}", b),
            AndError::Both(a, b) => write!(f, "{}; {}", a, b),
        }
    }
}

impl<A: std::error::Error + 'static, B: std::error::Error + 'static> std::error::Error
    for AndError<A, B>
{
}

// Implement Send + Sync for AndError when A and B are Send + Sync
unsafe impl<A: Send, B: Send> Send for AndError<A, B> {}
unsafe impl<A: Sync, B: Sync> Sync for AndError<A, B> {}

/// At least one predicate must hold
///
/// The `Or` combinator succeeds if either predicate passes.
/// It only fails if both predicates fail.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, Positive, Negative, Or};
///
/// // Value must be positive OR negative (i.e., non-zero)
/// type NonZeroAlt = Refined<i32, Or<Positive, Negative>>;
///
/// let p = NonZeroAlt::new(5).unwrap();
/// let n = NonZeroAlt::new(-5).unwrap();
/// assert!(NonZeroAlt::new(0).is_err());
/// ```
#[derive(Clone, Copy, Default)]
pub struct Or<A, B>(PhantomData<(A, B)>);

impl<A, B> fmt::Debug for Or<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Or<{}, {}>",
            std::any::type_name::<A>(),
            std::any::type_name::<B>()
        )
    }
}

impl<T, A, B> Predicate<T> for Or<A, B>
where
    A: Predicate<T>,
    B: Predicate<T>,
{
    type Error = OrError<A::Error, B::Error>;

    fn check(value: &T) -> Result<(), Self::Error> {
        match A::check(value) {
            Ok(()) => Ok(()),
            Err(a_err) => match B::check(value) {
                Ok(()) => Ok(()),
                Err(b_err) => Err(OrError(a_err, b_err)),
            },
        }
    }

    fn description() -> &'static str {
        "at least one predicate must hold"
    }
}

/// Error type for Or combinator (both predicates failed)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrError<A, B>(pub A, pub B);

impl<A: fmt::Display, B: fmt::Display> fmt::Display for OrError<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "neither predicate held: {} and {}", self.0, self.1)
    }
}

impl<A: std::error::Error + 'static, B: std::error::Error + 'static> std::error::Error
    for OrError<A, B>
{
}

// Implement Send + Sync for OrError when A and B are Send + Sync
unsafe impl<A: Send, B: Send> Send for OrError<A, B> {}
unsafe impl<A: Sync, B: Sync> Sync for OrError<A, B> {}

/// Predicate must NOT hold
///
/// The `Not` combinator inverts a predicate: it succeeds when the
/// inner predicate fails, and fails when the inner predicate succeeds.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, Positive, Not};
///
/// // Value must NOT be positive (i.e., <= 0)
/// type NotPositive = Refined<i32, Not<Positive>>;
///
/// let z = NotPositive::new(0).unwrap();
/// let n = NotPositive::new(-5).unwrap();
/// assert!(NotPositive::new(5).is_err());
/// ```
#[derive(Clone, Copy, Default)]
pub struct Not<A>(PhantomData<A>);

impl<A> fmt::Debug for Not<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not<{}>", std::any::type_name::<A>())
    }
}

impl<T, A: Predicate<T>> Predicate<T> for Not<A> {
    type Error = NotError;

    fn check(value: &T) -> Result<(), Self::Error> {
        match A::check(value) {
            Ok(()) => Err(NotError(A::description())),
            Err(_) => Ok(()),
        }
    }

    fn description() -> &'static str {
        "predicate must not hold"
    }
}

/// Error type for Not combinator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotError(pub &'static str);

impl fmt::Display for NotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "value must NOT satisfy: {}", self.0)
    }
}

impl std::error::Error for NotError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refined::predicates::numeric::{Negative, NonZero, Positive};
    use crate::refined::predicates::string::{NonEmpty, Trimmed};
    use crate::refined::Refined;

    #[test]
    fn test_and_both_pass() {
        type CleanString = Refined<String, And<NonEmpty, Trimmed>>;
        let result = CleanString::new("hello".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_and_first_fails() {
        type CleanString = Refined<String, And<NonEmpty, Trimmed>>;
        let result = CleanString::new("".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            AndError::First(_) | AndError::Both(_, _) => (),
            _ => panic!("Expected First or Both error"),
        }
    }

    #[test]
    fn test_and_second_fails() {
        type CleanString = Refined<String, And<NonEmpty, Trimmed>>;
        let result = CleanString::new("  hello  ".to_string());
        assert!(result.is_err());
        match result.unwrap_err() {
            AndError::Second(_) => (),
            _ => panic!("Expected Second error"),
        }
    }

    #[test]
    fn test_and_both_fail() {
        // Use Positive AND Negative - 0 fails both predicates
        type BothFail = Refined<i32, And<Positive, Negative>>;
        let result = BothFail::new(0);
        assert!(result.is_err());
        match result.unwrap_err() {
            AndError::Both(_, _) => (),
            _ => panic!("Expected Both error"),
        }
    }

    #[test]
    fn test_or_first_passes() {
        type NonZeroAlt = Refined<i32, Or<Positive, Negative>>;
        let result = NonZeroAlt::new(5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_or_second_passes() {
        type NonZeroAlt = Refined<i32, Or<Positive, Negative>>;
        let result = NonZeroAlt::new(-5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_or_both_fail() {
        type NonZeroAlt = Refined<i32, Or<Positive, Negative>>;
        let result = NonZeroAlt::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_not_inverts() {
        type NotPositive = Refined<i32, Not<Positive>>;
        assert!(NotPositive::new(0).is_ok());
        assert!(NotPositive::new(-5).is_ok());
        assert!(NotPositive::new(5).is_err());
    }

    #[test]
    fn test_not_error_message() {
        type NotPositive = Refined<i32, Not<Positive>>;
        let result = NotPositive::new(5);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.0.contains("positive"));
    }

    #[test]
    fn test_complex_composition() {
        // (NonEmpty AND Trimmed) OR has whitespace leading
        // Actually let's do: positive AND (nonzero OR something)
        type Complex = Refined<i32, And<Positive, Or<NonZero, Negative>>>;
        // Positive numbers that are nonzero pass
        assert!(Complex::new(5).is_ok());
        // 0 fails (not positive)
        assert!(Complex::new(0).is_err());
        // Negative fails (not positive)
        assert!(Complex::new(-5).is_err());
    }

    #[test]
    fn test_and_error_display() {
        let err: AndError<&str, &str> = AndError::First("first error");
        assert_eq!(format!("{}", err), "first error");

        let err: AndError<&str, &str> = AndError::Second("second error");
        assert_eq!(format!("{}", err), "second error");

        let err: AndError<&str, &str> = AndError::Both("first", "second");
        assert_eq!(format!("{}", err), "first; second");
    }

    #[test]
    fn test_or_error_display() {
        let err = OrError("first", "second");
        assert_eq!(
            format!("{}", err),
            "neither predicate held: first and second"
        );
    }

    #[test]
    fn test_not_error_display() {
        let err = NotError("positive number (> 0)");
        assert_eq!(
            format!("{}", err),
            "value must NOT satisfy: positive number (> 0)"
        );
    }
}
