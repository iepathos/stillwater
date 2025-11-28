//! String predicates
//!
//! This module provides common predicates for string validation.

use super::combinators::Predicate;

/// Predicate that checks if a string is not empty.
#[derive(Clone, Copy, Default, Debug)]
pub struct NotEmpty;

impl Predicate<str> for NotEmpty {
    #[inline]
    fn check(&self, value: &str) -> bool {
        !value.is_empty()
    }
}

impl Predicate<String> for NotEmpty {
    #[inline]
    fn check(&self, value: &String) -> bool {
        !value.is_empty()
    }
}

/// Create a predicate that checks if a string is not empty.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(not_empty().check("hello"));
/// assert!(!not_empty().check(""));
/// ```
pub fn not_empty() -> NotEmpty {
    NotEmpty
}

/// Predicate that checks string length is in range.
#[derive(Clone, Copy, Debug)]
pub struct LenBetween {
    min: usize,
    max: usize,
}

impl Predicate<str> for LenBetween {
    #[inline]
    fn check(&self, value: &str) -> bool {
        let len = value.len();
        len >= self.min && len <= self.max
    }
}

impl Predicate<String> for LenBetween {
    #[inline]
    fn check(&self, value: &String) -> bool {
        let len = value.len();
        len >= self.min && len <= self.max
    }
}

/// Create a predicate that checks if string length is between min and max (inclusive).
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// let p = len_between(3, 10);
/// assert!(!p.check("ab"));      // too short
/// assert!(p.check("abc"));      // exactly min
/// assert!(p.check("hello"));    // in range
/// assert!(p.check("1234567890")); // exactly max
/// assert!(!p.check("12345678901")); // too long
/// ```
pub fn len_between(min: usize, max: usize) -> LenBetween {
    LenBetween { min, max }
}

/// Create a predicate that checks if string length is at least min.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(len_min(3).check("hello"));
/// assert!(len_min(3).check("abc"));
/// assert!(!len_min(3).check("ab"));
/// ```
pub fn len_min(min: usize) -> LenBetween {
    LenBetween {
        min,
        max: usize::MAX,
    }
}

/// Create a predicate that checks if string length is at most max.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(len_max(5).check("hello"));
/// assert!(len_max(5).check("hi"));
/// assert!(!len_max(5).check("toolong"));
/// ```
pub fn len_max(max: usize) -> LenBetween {
    LenBetween { min: 0, max }
}

/// Create a predicate that checks if string length is exactly len.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(len_eq(5).check("hello"));
/// assert!(!len_eq(5).check("hi"));
/// ```
pub fn len_eq(len: usize) -> LenBetween {
    LenBetween { min: len, max: len }
}

/// Predicate that checks if string starts with a prefix.
#[derive(Clone, Debug)]
pub struct StartsWith<S>(pub S);

impl<S: AsRef<str> + Send + Sync> Predicate<str> for StartsWith<S> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.starts_with(self.0.as_ref())
    }
}

impl<S: AsRef<str> + Send + Sync> Predicate<String> for StartsWith<S> {
    #[inline]
    fn check(&self, value: &String) -> bool {
        value.starts_with(self.0.as_ref())
    }
}

/// Create a predicate that checks if string starts with prefix.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(starts_with("http").check("https://example.com"));
/// assert!(!starts_with("http").check("ftp://example.com"));
/// ```
pub fn starts_with<S: AsRef<str> + Send + Sync>(prefix: S) -> StartsWith<S> {
    StartsWith(prefix)
}

/// Predicate that checks if string ends with a suffix.
#[derive(Clone, Debug)]
pub struct EndsWith<S>(pub S);

impl<S: AsRef<str> + Send + Sync> Predicate<str> for EndsWith<S> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.ends_with(self.0.as_ref())
    }
}

impl<S: AsRef<str> + Send + Sync> Predicate<String> for EndsWith<S> {
    #[inline]
    fn check(&self, value: &String) -> bool {
        value.ends_with(self.0.as_ref())
    }
}

/// Create a predicate that checks if string ends with suffix.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(ends_with(".rs").check("main.rs"));
/// assert!(!ends_with(".rs").check("main.py"));
/// ```
pub fn ends_with<S: AsRef<str> + Send + Sync>(suffix: S) -> EndsWith<S> {
    EndsWith(suffix)
}

/// Predicate that checks if string contains a substring.
#[derive(Clone, Debug)]
pub struct Contains<S>(pub S);

impl<S: AsRef<str> + Send + Sync> Predicate<str> for Contains<S> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.contains(self.0.as_ref())
    }
}

impl<S: AsRef<str> + Send + Sync> Predicate<String> for Contains<S> {
    #[inline]
    fn check(&self, value: &String) -> bool {
        value.contains(self.0.as_ref())
    }
}

/// Create a predicate that checks if string contains substring.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(contains("@").check("user@example.com"));
/// assert!(!contains("@").check("invalid"));
/// ```
pub fn contains<S: AsRef<str> + Send + Sync>(substring: S) -> Contains<S> {
    Contains(substring)
}

/// Predicate that checks if all characters satisfy a predicate.
#[derive(Clone, Copy, Debug)]
pub struct AllChars<F>(pub F);

impl<F: Fn(char) -> bool + Send + Sync> Predicate<str> for AllChars<F> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.chars().all(&self.0)
    }
}

impl<F: Fn(char) -> bool + Send + Sync> Predicate<String> for AllChars<F> {
    #[inline]
    fn check(&self, value: &String) -> bool {
        value.chars().all(&self.0)
    }
}

/// Create a predicate that checks if all characters satisfy a condition.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(all_chars(char::is_alphabetic).check("hello"));
/// assert!(!all_chars(char::is_alphabetic).check("hello123"));
/// ```
pub fn all_chars<F: Fn(char) -> bool + Send + Sync>(f: F) -> AllChars<F> {
    AllChars(f)
}

/// Predicate that checks if any character satisfies a predicate.
#[derive(Clone, Copy, Debug)]
pub struct AnyChar<F>(pub F);

impl<F: Fn(char) -> bool + Send + Sync> Predicate<str> for AnyChar<F> {
    #[inline]
    fn check(&self, value: &str) -> bool {
        value.chars().any(&self.0)
    }
}

impl<F: Fn(char) -> bool + Send + Sync> Predicate<String> for AnyChar<F> {
    #[inline]
    fn check(&self, value: &String) -> bool {
        value.chars().any(&self.0)
    }
}

/// Create a predicate that checks if any character satisfies a condition.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(any_char(char::is_numeric).check("hello123"));
/// assert!(!any_char(char::is_numeric).check("hello"));
/// ```
pub fn any_char<F: Fn(char) -> bool + Send + Sync>(f: F) -> AnyChar<F> {
    AnyChar(f)
}

/// Create a predicate that checks if all characters are ASCII.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(is_ascii().check("hello"));
/// assert!(!is_ascii().check("héllo"));
/// ```
pub fn is_ascii() -> AllChars<fn(char) -> bool> {
    AllChars(|c| c.is_ascii())
}

/// Create a predicate that checks if all characters are alphanumeric.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(is_alphanumeric().check("hello123"));
/// assert!(!is_alphanumeric().check("hello_123"));
/// ```
pub fn is_alphanumeric() -> AllChars<fn(char) -> bool> {
    AllChars(|c| c.is_alphanumeric())
}

/// Create a predicate that checks if all characters are alphabetic.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(is_alphabetic().check("hello"));
/// assert!(!is_alphabetic().check("hello123"));
/// ```
pub fn is_alphabetic() -> AllChars<fn(char) -> bool> {
    AllChars(|c| c.is_alphabetic())
}

/// Create a predicate that checks if all characters are numeric.
///
/// # Example
///
/// ```rust
/// use stillwater::predicate::*;
///
/// assert!(is_numeric().check("123"));
/// assert!(!is_numeric().check("hello123"));
/// ```
pub fn is_numeric() -> AllChars<fn(char) -> bool> {
    AllChars(|c| c.is_numeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{And, PredicateExt};

    #[test]
    fn test_not_empty() {
        assert!(not_empty().check("hello"));
        assert!(!not_empty().check(""));
    }

    #[test]
    fn test_not_empty_string() {
        assert!(not_empty().check(&String::from("hello")));
        assert!(!not_empty().check(&String::new()));
    }

    #[test]
    fn test_len_between() {
        let p = len_between(3, 10);
        assert!(!p.check("ab")); // too short
        assert!(p.check("abc")); // exactly min
        assert!(p.check("hello")); // in range
        assert!(p.check("1234567890")); // exactly max
        assert!(!p.check("12345678901")); // too long
    }

    #[test]
    fn test_len_min() {
        assert!(len_min(3).check("hello"));
        assert!(len_min(3).check("abc"));
        assert!(!len_min(3).check("ab"));
    }

    #[test]
    fn test_len_max() {
        assert!(len_max(5).check("hello"));
        assert!(len_max(5).check("hi"));
        assert!(!len_max(5).check("toolong"));
    }

    #[test]
    fn test_len_eq() {
        assert!(len_eq(5).check("hello"));
        assert!(!len_eq(5).check("hi"));
        assert!(!len_eq(5).check("toolong"));
    }

    #[test]
    fn test_starts_with() {
        assert!(starts_with("http").check("https://example.com"));
        assert!(!starts_with("http").check("ftp://example.com"));
    }

    #[test]
    fn test_ends_with() {
        assert!(ends_with(".rs").check("main.rs"));
        assert!(!ends_with(".rs").check("main.py"));
    }

    #[test]
    fn test_contains() {
        assert!(contains("@").check("user@example.com"));
        assert!(!contains("@").check("invalid"));
    }

    #[test]
    fn test_all_chars() {
        assert!(all_chars(char::is_alphabetic).check("hello"));
        assert!(!all_chars(char::is_alphabetic).check("hello123"));
    }

    #[test]
    fn test_any_char() {
        assert!(any_char(char::is_numeric).check("hello123"));
        assert!(!any_char(char::is_numeric).check("hello"));
    }

    #[test]
    fn test_is_ascii() {
        assert!(is_ascii().check("hello"));
        assert!(!is_ascii().check("héllo"));
    }

    #[test]
    fn test_is_alphanumeric() {
        assert!(is_alphanumeric().check("hello123"));
        assert!(!is_alphanumeric().check("hello_123"));
    }

    #[test]
    fn test_is_alphabetic() {
        assert!(is_alphabetic().check("hello"));
        assert!(!is_alphabetic().check("hello123"));
    }

    #[test]
    fn test_is_numeric() {
        assert!(is_numeric().check("123"));
        assert!(!is_numeric().check("hello123"));
    }

    #[test]
    fn test_complex_username_validation() {
        // For combined predicates on strings, we use type annotation to specify str
        // This tells the compiler which Predicate impl to use
        let valid_username: And<And<NotEmpty, LenBetween>, AllChars<_>> = PredicateExt::<str>::and(
            PredicateExt::<str>::and(not_empty(), len_between(3, 20)),
            all_chars(|c: char| c.is_alphanumeric() || c == '_'),
        );

        // Call check on &str literals
        assert!(valid_username.check("john_doe"));
        assert!(valid_username.check("a_1"));
        assert!(!valid_username.check("ab")); // too short
        assert!(!valid_username.check("invalid-name")); // contains hyphen
    }
}
