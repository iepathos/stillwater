//! Type aliases for common refined types
//!
//! This module provides convenient type aliases for commonly used
//! refined type combinations.
//!
//! # Example
//!
//! ```rust
//! use stillwater::refined::{NonEmptyString, PositiveI32, NonNegativeI64};
//!
//! let name = NonEmptyString::new("Alice".to_string()).unwrap();
//! let age = PositiveI32::new(25).unwrap();
//! let balance = NonNegativeI64::new(1000).unwrap();
//! ```

use super::combinators::And;
use super::predicates::collection::MaxSize;
use super::predicates::numeric::{InRange, Negative, NonNegative, NonZero, Positive};
use super::predicates::string::{MaxLength, MinLength, NonEmpty, Trimmed};
use super::Refined;

// ============================================================================
// String aliases
// ============================================================================

/// A string that is guaranteed to be non-empty
pub type NonEmptyString = Refined<String, NonEmpty>;

/// A string that is guaranteed to have no leading/trailing whitespace
pub type TrimmedString = Refined<String, Trimmed>;

/// A string that is both non-empty and trimmed
pub type NonEmptyTrimmedString = Refined<String, And<NonEmpty, Trimmed>>;

// ============================================================================
// Signed integer aliases - Positive
// ============================================================================

/// An i8 that is guaranteed to be positive (> 0)
pub type PositiveI8 = Refined<i8, Positive>;

/// An i16 that is guaranteed to be positive (> 0)
pub type PositiveI16 = Refined<i16, Positive>;

/// An i32 that is guaranteed to be positive (> 0)
pub type PositiveI32 = Refined<i32, Positive>;

/// An i64 that is guaranteed to be positive (> 0)
pub type PositiveI64 = Refined<i64, Positive>;

/// An i128 that is guaranteed to be positive (> 0)
pub type PositiveI128 = Refined<i128, Positive>;

/// An isize that is guaranteed to be positive (> 0)
pub type PositiveIsize = Refined<isize, Positive>;

// ============================================================================
// Signed integer aliases - NonNegative
// ============================================================================

/// An i8 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeI8 = Refined<i8, NonNegative>;

/// An i16 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeI16 = Refined<i16, NonNegative>;

/// An i32 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeI32 = Refined<i32, NonNegative>;

/// An i64 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeI64 = Refined<i64, NonNegative>;

/// An i128 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeI128 = Refined<i128, NonNegative>;

/// An isize that is guaranteed to be non-negative (>= 0)
pub type NonNegativeIsize = Refined<isize, NonNegative>;

// ============================================================================
// Signed integer aliases - Negative
// ============================================================================

/// An i8 that is guaranteed to be negative (< 0)
pub type NegativeI8 = Refined<i8, Negative>;

/// An i16 that is guaranteed to be negative (< 0)
pub type NegativeI16 = Refined<i16, Negative>;

/// An i32 that is guaranteed to be negative (< 0)
pub type NegativeI32 = Refined<i32, Negative>;

/// An i64 that is guaranteed to be negative (< 0)
pub type NegativeI64 = Refined<i64, Negative>;

/// An i128 that is guaranteed to be negative (< 0)
pub type NegativeI128 = Refined<i128, Negative>;

/// An isize that is guaranteed to be negative (< 0)
pub type NegativeIsize = Refined<isize, Negative>;

// ============================================================================
// Signed integer aliases - NonZero
// ============================================================================

/// An i8 that is guaranteed to be non-zero (!= 0)
pub type NonZeroI8 = Refined<i8, NonZero>;

/// An i16 that is guaranteed to be non-zero (!= 0)
pub type NonZeroI16 = Refined<i16, NonZero>;

/// An i32 that is guaranteed to be non-zero (!= 0)
pub type NonZeroI32 = Refined<i32, NonZero>;

/// An i64 that is guaranteed to be non-zero (!= 0)
pub type NonZeroI64 = Refined<i64, NonZero>;

/// An i128 that is guaranteed to be non-zero (!= 0)
pub type NonZeroI128 = Refined<i128, NonZero>;

/// An isize that is guaranteed to be non-zero (!= 0)
pub type NonZeroIsize = Refined<isize, NonZero>;

// ============================================================================
// Unsigned integer aliases - NonZero
// (Unsigned integers are always non-negative, so only NonZero makes sense)
// ============================================================================

/// A u8 that is guaranteed to be non-zero (!= 0)
pub type NonZeroU8 = Refined<u8, NonZero>;

/// A u16 that is guaranteed to be non-zero (!= 0)
pub type NonZeroU16 = Refined<u16, NonZero>;

/// A u32 that is guaranteed to be non-zero (!= 0)
pub type NonZeroU32 = Refined<u32, NonZero>;

/// A u64 that is guaranteed to be non-zero (!= 0)
pub type NonZeroU64 = Refined<u64, NonZero>;

/// A u128 that is guaranteed to be non-zero (!= 0)
pub type NonZeroU128 = Refined<u128, NonZero>;

/// A usize that is guaranteed to be non-zero (!= 0)
pub type NonZeroUsize = Refined<usize, NonZero>;

// ============================================================================
// Float aliases
// ============================================================================

/// An f32 that is guaranteed to be positive (> 0)
pub type PositiveF32 = Refined<f32, Positive>;

/// An f64 that is guaranteed to be positive (> 0)
pub type PositiveF64 = Refined<f64, Positive>;

/// An f32 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeF32 = Refined<f32, NonNegative>;

/// An f64 that is guaranteed to be non-negative (>= 0)
pub type NonNegativeF64 = Refined<f64, NonNegative>;

/// An f32 that is guaranteed to be negative (< 0)
pub type NegativeF32 = Refined<f32, Negative>;

/// An f64 that is guaranteed to be negative (< 0)
pub type NegativeF64 = Refined<f64, Negative>;

// ============================================================================
// Collection aliases
// Note: Using NonEmptyList to avoid conflict with existing NonEmptyVec
// ============================================================================

/// A `Vec<T>` that is guaranteed to be non-empty
///
/// Note: This is different from `stillwater::NonEmptyVec<T>` which has
/// a specialized API. Use `NonEmptyList` for the generic refined type pattern.
pub type NonEmptyList<T> = Refined<Vec<T>, NonEmpty>;

// ============================================================================
// Common domain aliases
// ============================================================================

/// A percentage value (0-100 inclusive)
pub type Percentage = Refined<i32, InRange<0, 100>>;

/// A network port number (1-65535)
pub type Port = Refined<u16, InRange<1, 65535>>;

/// A bounded string with maximum length
pub type BoundedString<const MAX: usize> = Refined<String, MaxLength<MAX>>;

/// A bounded string with minimum length
pub type MinLengthString<const MIN: usize> = Refined<String, MinLength<MIN>>;

/// A bounded collection with maximum size
pub type BoundedVec<T, const MAX: usize> = Refined<Vec<T>, MaxSize<MAX>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_aliases() {
        assert!(NonEmptyString::new("hello".to_string()).is_ok());
        assert!(NonEmptyString::new("".to_string()).is_err());

        assert!(TrimmedString::new("hello".to_string()).is_ok());
        assert!(TrimmedString::new("  hello  ".to_string()).is_err());

        assert!(NonEmptyTrimmedString::new("hello".to_string()).is_ok());
        assert!(NonEmptyTrimmedString::new("".to_string()).is_err());
        assert!(NonEmptyTrimmedString::new("  hello  ".to_string()).is_err());
    }

    #[test]
    fn test_positive_aliases() {
        assert!(PositiveI32::new(1).is_ok());
        assert!(PositiveI32::new(0).is_err());
        assert!(PositiveI32::new(-1).is_err());

        assert!(PositiveF64::new(0.1).is_ok());
        assert!(PositiveF64::new(0.0).is_err());
    }

    #[test]
    fn test_non_negative_aliases() {
        assert!(NonNegativeI32::new(0).is_ok());
        assert!(NonNegativeI32::new(1).is_ok());
        assert!(NonNegativeI32::new(-1).is_err());
    }

    #[test]
    fn test_negative_aliases() {
        assert!(NegativeI32::new(-1).is_ok());
        assert!(NegativeI32::new(0).is_err());
        assert!(NegativeI32::new(1).is_err());
    }

    #[test]
    fn test_non_zero_aliases() {
        assert!(NonZeroI32::new(1).is_ok());
        assert!(NonZeroI32::new(-1).is_ok());
        assert!(NonZeroI32::new(0).is_err());

        assert!(NonZeroU32::new(1).is_ok());
        assert!(NonZeroU32::new(0).is_err());
    }

    #[test]
    fn test_collection_aliases() {
        assert!(NonEmptyList::<i32>::new(vec![1]).is_ok());
        assert!(NonEmptyList::<i32>::new(vec![]).is_err());
    }

    #[test]
    fn test_domain_aliases() {
        // Percentage
        assert!(Percentage::new(0).is_ok());
        assert!(Percentage::new(100).is_ok());
        assert!(Percentage::new(50).is_ok());
        assert!(Percentage::new(-1).is_err());
        assert!(Percentage::new(101).is_err());

        // Port
        assert!(Port::new(80).is_ok());
        assert!(Port::new(443).is_ok());
        assert!(Port::new(1).is_ok());
        assert!(Port::new(65535).is_ok());
        assert!(Port::new(0).is_err());
    }

    #[test]
    fn test_bounded_string() {
        type ShortString = BoundedString<10>;
        assert!(ShortString::new("hello".to_string()).is_ok());
        assert!(ShortString::new("this is too long".to_string()).is_err());
    }

    #[test]
    fn test_bounded_vec() {
        type SmallVec = BoundedVec<i32, 3>;
        assert!(SmallVec::new(vec![1, 2, 3]).is_ok());
        assert!(SmallVec::new(vec![1, 2, 3, 4]).is_err());
    }
}
