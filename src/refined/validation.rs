//! Validation integration for refined types
//!
//! This module provides integration between refined types and the
//! [`Validation`] type for error accumulation.
//!
//! # Example
//!
//! ```rust
//! use stillwater::{Validation, refined::{Refined, NonEmpty, Positive}};
//!
//! type NonEmptyString = Refined<String, NonEmpty>;
//! type PositiveI32 = Refined<i32, Positive>;
//!
//! // Validate multiple fields, accumulating errors
//! fn validate_user(name: String, age: i32) -> Validation<(NonEmptyString, PositiveI32), Vec<&'static str>> {
//!     let v1 = NonEmptyString::validate_vec(name);
//!     let v2 = PositiveI32::validate_vec(age);
//!     v1.and(v2)
//! }
//!
//! // All errors collected
//! let result = validate_user("".to_string(), -5);
//! assert!(result.is_failure());
//! ```

use std::fmt;

use super::{Predicate, Refined};
use crate::Validation;

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Validate a value, returning a Validation result.
    ///
    /// Use this with `Validation::all` for error accumulation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::{Validation, refined::{Refined, Positive}};
    ///
    /// type PositiveI32 = Refined<i32, Positive>;
    ///
    /// let valid = PositiveI32::validate(42);
    /// assert!(valid.is_success());
    ///
    /// let invalid = PositiveI32::validate(-5);
    /// assert!(invalid.is_failure());
    /// ```
    pub fn validate(value: T) -> Validation<Self, P::Error> {
        match Self::new(value) {
            Ok(refined) => Validation::Success(refined),
            Err(e) => Validation::Failure(e),
        }
    }

    /// Validate a value, wrapping the error in a Vec for accumulation.
    ///
    /// This is useful when combining with other validations that
    /// produce `Vec<E>` errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::{Validation, refined::{Refined, NonEmpty, Positive}};
    ///
    /// type NonEmptyString = Refined<String, NonEmpty>;
    /// type PositiveI32 = Refined<i32, Positive>;
    ///
    /// let v1 = NonEmptyString::validate_vec("alice".to_string());
    /// let v2 = PositiveI32::validate_vec(25);
    /// let result = v1.and(v2);
    /// assert!(result.is_success());
    /// ```
    pub fn validate_vec(value: T) -> Validation<Self, Vec<P::Error>> {
        match Self::new(value) {
            Ok(refined) => Validation::Success(refined),
            Err(e) => Validation::Failure(vec![e]),
        }
    }
}

/// Error with field context
///
/// Wraps an error with a field name for better error messages.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::FieldError;
///
/// let err = FieldError {
///     field: "username",
///     error: "cannot be empty",
/// };
/// assert_eq!(format!("{}", err), "username: cannot be empty");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError<E> {
    /// The field name
    pub field: &'static str,
    /// The underlying error
    pub error: E,
}

impl<E: fmt::Display> fmt::Display for FieldError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.error)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for FieldError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Extension trait for creating refined validations with field context
pub trait RefinedValidationExt<T, P: Predicate<T>> {
    /// Validate with a field name for error context
    fn validate_field(
        value: T,
        field: &'static str,
    ) -> Validation<Refined<T, P>, FieldError<P::Error>>;
}

impl<T, P: Predicate<T>> RefinedValidationExt<T, P> for Refined<T, P> {
    fn validate_field(
        value: T,
        field: &'static str,
    ) -> Validation<Refined<T, P>, FieldError<P::Error>> {
        match Refined::new(value) {
            Ok(refined) => Validation::Success(refined),
            Err(e) => Validation::Failure(FieldError { field, error: e }),
        }
    }
}

/// Extension trait for adding field context to validations
pub trait ValidationFieldExt<T, E> {
    /// Add field context to a validation error
    fn with_field(self, field: &'static str) -> Validation<T, FieldError<E>>;
}

impl<T, E> ValidationFieldExt<T, E> for Validation<T, E> {
    /// Add field context to a validation error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stillwater::{Validation, refined::{Refined, NonEmpty, ValidationFieldExt}};
    ///
    /// type NonEmptyString = Refined<String, NonEmpty>;
    ///
    /// let result = NonEmptyString::validate("".to_string())
    ///     .with_field("username");
    ///
    /// match result {
    ///     Validation::Failure(err) => {
    ///         assert_eq!(err.field, "username");
    ///     }
    ///     _ => panic!("Expected failure"),
    /// }
    /// ```
    fn with_field(self, field: &'static str) -> Validation<T, FieldError<E>> {
        match self {
            Validation::Success(v) => Validation::Success(v),
            Validation::Failure(e) => Validation::Failure(FieldError { field, error: e }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refined::predicates::numeric::Positive;
    use crate::refined::predicates::string::NonEmpty;

    type NonEmptyString = Refined<String, NonEmpty>;
    type PositiveI32 = Refined<i32, Positive>;

    #[test]
    fn test_validate_success() {
        let result = PositiveI32::validate(42);
        assert!(result.is_success());
    }

    #[test]
    fn test_validate_failure() {
        let result = PositiveI32::validate(-5);
        assert!(result.is_failure());
    }

    #[test]
    fn test_validate_vec_success() {
        let result = NonEmptyString::validate_vec("hello".to_string());
        assert!(result.is_success());
    }

    #[test]
    fn test_validate_vec_failure() {
        let result = NonEmptyString::validate_vec("".to_string());
        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 1);
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_validate_all() {
        let v1 = NonEmptyString::validate_vec("alice".to_string());
        let v2 = PositiveI32::validate_vec(25);
        let result = v1.and(v2);
        assert!(result.is_success());
    }

    #[test]
    fn test_validate_all_with_errors() {
        let v1 = NonEmptyString::validate_vec("".to_string());
        let v2 = PositiveI32::validate_vec(-5);
        let result = v1.and(v2);
        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
            }
            _ => panic!("Expected failure with 2 errors"),
        }
    }

    #[test]
    fn test_validate_field() {
        let result = NonEmptyString::validate_field("".to_string(), "username");
        match result {
            Validation::Failure(err) => {
                assert_eq!(err.field, "username");
                assert_eq!(err.error, "string cannot be empty");
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_with_field() {
        let result = NonEmptyString::validate("".to_string()).with_field("username");
        match result {
            Validation::Failure(err) => {
                assert_eq!(err.field, "username");
            }
            _ => panic!("Expected failure"),
        }
    }

    #[test]
    fn test_field_error_display() {
        let err = FieldError {
            field: "username",
            error: "cannot be empty",
        };
        assert_eq!(format!("{}", err), "username: cannot be empty");
    }

    #[test]
    fn test_combined_field_validation() {
        let v1 = NonEmptyString::validate("".to_string())
            .with_field("name")
            .map_err(|e| vec![e]);
        let v2 = PositiveI32::validate(-5)
            .with_field("age")
            .map_err(|e| vec![e]);
        let result: Validation<(NonEmptyString, PositiveI32), Vec<FieldError<&str>>> = v1.and(v2);

        match result {
            Validation::Failure(errors) => {
                assert_eq!(errors.len(), 2);
                assert_eq!(errors[0].field, "name");
                assert_eq!(errors[1].field, "age");
            }
            _ => panic!("Expected failure with 2 errors"),
        }
    }
}
