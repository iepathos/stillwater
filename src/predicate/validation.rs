//! Validation integration for predicates
//!
//! This module provides integration between predicates and the `Validation` type.

use super::combinators::Predicate;
use crate::Validation;

/// Validate a value using a predicate.
///
/// Returns `Validation::success(value)` if the predicate is satisfied,
/// otherwise returns `Validation::failure(error)`.
///
/// # Example
///
/// ```rust
/// use stillwater::{Validation, predicate::*};
///
/// let result = validate(String::from("hello"), len_min(3), "too short");
/// assert_eq!(result, Validation::success(String::from("hello")));
///
/// let result = validate(String::from("hi"), len_min(3), "too short");
/// assert_eq!(result, Validation::failure("too short"));
/// ```
pub fn validate<T, E, P>(value: T, predicate: P, error: E) -> Validation<T, E>
where
    P: Predicate<T>,
{
    if predicate.check(&value) {
        Validation::success(value)
    } else {
        Validation::failure(error)
    }
}

/// Validate a value with an error factory.
///
/// Like `validate`, but takes a closure to generate the error,
/// allowing access to the value when constructing the error message.
///
/// # Example
///
/// ```rust
/// use stillwater::{Validation, predicate::*};
///
/// let result = validate_with(
///     String::from("hi"),
///     len_min(3),
///     |s| format!("'{}' is too short (min 3 chars)", s)
/// );
/// assert_eq!(result, Validation::failure("'hi' is too short (min 3 chars)".to_string()));
/// ```
pub fn validate_with<T, E, P, F>(value: T, predicate: P, error_fn: F) -> Validation<T, E>
where
    P: Predicate<T>,
    F: FnOnce(&T) -> E,
{
    if predicate.check(&value) {
        Validation::success(value)
    } else {
        Validation::failure(error_fn(&value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{len_max, len_min};

    #[test]
    fn test_validate_success() {
        // Use String to match Predicate<String> implementation
        let result = validate(String::from("hello"), len_min(3), "too short");
        assert_eq!(result, Validation::success(String::from("hello")));
    }

    #[test]
    fn test_validate_failure() {
        let result = validate(String::from("hi"), len_min(3), "too short");
        assert_eq!(result, Validation::failure("too short"));
    }

    #[test]
    fn test_validate_with_success() {
        let result = validate_with(String::from("hello"), len_min(3), |s| {
            format!("'{}' too short", s)
        });
        assert_eq!(result, Validation::success(String::from("hello")));
    }

    #[test]
    fn test_validate_with_failure() {
        let result = validate_with(String::from("hi"), len_min(3), |s| {
            format!("'{}' too short", s)
        });
        assert_eq!(result, Validation::failure("'hi' too short".to_string()));
    }

    #[test]
    fn test_validate_with_numbers() {
        use crate::predicate::{between, positive};

        let result = validate(42, positive::<i32>(), "must be positive");
        assert_eq!(result, Validation::success(42));

        let result = validate(-5, positive::<i32>(), "must be positive");
        assert_eq!(result, Validation::failure("must be positive"));

        let result = validate(50, between(0, 100), "must be in range");
        assert_eq!(result, Validation::success(50));
    }

    #[test]
    fn test_validate_with_collections() {
        use crate::predicate::{has_min_len, is_not_empty};

        let result: Validation<Vec<i32>, &str> =
            validate(vec![1, 2, 3], is_not_empty(), "must not be empty");
        assert_eq!(result, Validation::success(vec![1, 2, 3]));

        let result: Validation<Vec<i32>, &str> =
            validate(vec![], is_not_empty(), "must not be empty");
        assert_eq!(result, Validation::failure("must not be empty"));

        let result: Validation<Vec<i32>, &str> =
            validate(vec![1], has_min_len(2), "need at least 2");
        assert_eq!(result, Validation::failure("need at least 2"));
    }

    #[test]
    fn test_validate_chain() {
        use crate::predicate::{And, LenBetween, PredicateExt};

        // Chain multiple predicates - use String
        // Type annotation needed because LenBetween implements both Predicate<str> and Predicate<String>
        let valid_username: And<LenBetween, LenBetween> =
            PredicateExt::<String>::and(len_min(3), len_max(20));

        let result = validate(String::from("john"), valid_username, "invalid username");
        assert_eq!(result, Validation::success(String::from("john")));
    }
}
