//! String predicates for refined types
//!
//! This module provides predicates for constraining string values:
//! - [`NonEmpty`]: String is not empty
//! - [`Trimmed`]: String has no leading/trailing whitespace
//! - [`MaxLength<N>`]: String length <= N
//! - [`MinLength<N>`]: String length >= N
//!
//! # Example
//!
//! ```rust
//! use stillwater::refined::{Refined, NonEmpty, Trimmed, MaxLength};
//! use stillwater::refined::And;
//!
//! // Non-empty string
//! type NonEmptyString = Refined<String, NonEmpty>;
//! let name = NonEmptyString::new("Alice".to_string()).unwrap();
//! assert!(NonEmptyString::new("".to_string()).is_err());
//!
//! // Trimmed string
//! type TrimmedString = Refined<String, Trimmed>;
//! let clean = TrimmedString::new("hello".to_string()).unwrap();
//! assert!(TrimmedString::new("  hello  ".to_string()).is_err());
//!
//! // Combined predicates
//! type Username = Refined<String, And<NonEmpty, MaxLength<20>>>;
//! let user = Username::new("alice".to_string()).unwrap();
//! ```

use super::super::Predicate;

/// String must not be empty
///
/// This predicate works for both `String` and `&str`.
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, NonEmpty};
///
/// type NonEmptyString = Refined<String, NonEmpty>;
///
/// let name = NonEmptyString::new("Alice".to_string()).unwrap();
/// assert!(NonEmptyString::new("".to_string()).is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NonEmpty;

impl Predicate<String> for NonEmpty {
    type Error = &'static str;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err("string cannot be empty")
        } else {
            Ok(())
        }
    }

    fn description() -> &'static str {
        "non-empty string"
    }
}

impl Predicate<&str> for NonEmpty {
    type Error = &'static str;

    fn check(value: &&str) -> Result<(), Self::Error> {
        if value.is_empty() {
            Err("string cannot be empty")
        } else {
            Ok(())
        }
    }

    fn description() -> &'static str {
        "non-empty string"
    }
}

/// String equals its trimmed form (no leading/trailing whitespace)
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, Trimmed};
///
/// type TrimmedString = Refined<String, Trimmed>;
///
/// let clean = TrimmedString::new("hello".to_string()).unwrap();
/// assert!(TrimmedString::new("  hello".to_string()).is_err());
/// assert!(TrimmedString::new("hello  ".to_string()).is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct Trimmed;

impl Predicate<String> for Trimmed {
    type Error = &'static str;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.trim() == value {
            Ok(())
        } else {
            Err("string has leading or trailing whitespace")
        }
    }

    fn description() -> &'static str {
        "trimmed string (no leading/trailing whitespace)"
    }
}

/// String length must be at most N bytes
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, MaxLength};
///
/// type ShortString = Refined<String, MaxLength<10>>;
///
/// let s = ShortString::new("hello".to_string()).unwrap();
/// assert!(ShortString::new("this is too long".to_string()).is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MaxLength<const N: usize>;

impl<const N: usize> Predicate<String> for MaxLength<N> {
    type Error = String;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.len() <= N {
            Ok(())
        } else {
            Err(format!(
                "string length {} exceeds maximum {}",
                value.len(),
                N
            ))
        }
    }

    fn description() -> &'static str {
        "string with maximum length"
    }
}

/// String length must be at least N bytes
///
/// # Example
///
/// ```rust
/// use stillwater::refined::{Refined, MinLength};
///
/// type LongEnough = Refined<String, MinLength<3>>;
///
/// let s = LongEnough::new("hello".to_string()).unwrap();
/// assert!(LongEnough::new("hi".to_string()).is_err());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct MinLength<const N: usize>;

impl<const N: usize> Predicate<String> for MinLength<N> {
    type Error = String;

    fn check(value: &String) -> Result<(), Self::Error> {
        if value.len() >= N {
            Ok(())
        } else {
            Err(format!(
                "string length {} is less than minimum {}",
                value.len(),
                N
            ))
        }
    }

    fn description() -> &'static str {
        "string with minimum length"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refined::Refined;

    type NonEmptyString = Refined<String, NonEmpty>;
    type TrimmedString = Refined<String, Trimmed>;
    type ShortString = Refined<String, MaxLength<10>>;
    type LongEnough = Refined<String, MinLength<3>>;

    #[test]
    fn test_non_empty_success() {
        assert!(NonEmptyString::new("a".to_string()).is_ok());
        assert!(NonEmptyString::new("hello".to_string()).is_ok());
        assert!(NonEmptyString::new(" ".to_string()).is_ok()); // whitespace is not empty
    }

    #[test]
    fn test_non_empty_failure() {
        let result = NonEmptyString::new("".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "string cannot be empty");
    }

    #[test]
    fn test_trimmed_success() {
        assert!(TrimmedString::new("hello".to_string()).is_ok());
        assert!(TrimmedString::new("hello world".to_string()).is_ok());
        assert!(TrimmedString::new("".to_string()).is_ok()); // empty is trimmed
    }

    #[test]
    fn test_trimmed_failure() {
        assert!(TrimmedString::new(" hello".to_string()).is_err());
        assert!(TrimmedString::new("hello ".to_string()).is_err());
        assert!(TrimmedString::new("  hello  ".to_string()).is_err());
        assert!(TrimmedString::new("\thello".to_string()).is_err());
        assert!(TrimmedString::new("hello\n".to_string()).is_err());
    }

    #[test]
    fn test_max_length_success() {
        assert!(ShortString::new("".to_string()).is_ok());
        assert!(ShortString::new("hello".to_string()).is_ok());
        assert!(ShortString::new("1234567890".to_string()).is_ok()); // exactly 10
    }

    #[test]
    fn test_max_length_failure() {
        let result = ShortString::new("12345678901".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_min_length_success() {
        assert!(LongEnough::new("abc".to_string()).is_ok()); // exactly 3
        assert!(LongEnough::new("hello".to_string()).is_ok());
    }

    #[test]
    fn test_min_length_failure() {
        assert!(LongEnough::new("".to_string()).is_err());
        assert!(LongEnough::new("ab".to_string()).is_err());
    }

    #[test]
    fn test_descriptions() {
        assert_eq!(
            <NonEmpty as Predicate<String>>::description(),
            "non-empty string"
        );
        assert_eq!(
            <Trimmed as Predicate<String>>::description(),
            "trimmed string (no leading/trailing whitespace)"
        );
        assert_eq!(
            <MaxLength<10> as Predicate<String>>::description(),
            "string with maximum length"
        );
        assert_eq!(
            <MinLength<3> as Predicate<String>>::description(),
            "string with minimum length"
        );
    }
}
