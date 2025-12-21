//! Numeric predicates for refined types
//!
//! This module provides predicates for constraining numeric values:
//! - [`Positive`]: Value > 0
//! - [`NonNegative`]: Value >= 0
//! - [`Negative`]: Value < 0
//! - [`NonZero`]: Value != 0
//! - [`InRange<MIN, MAX>`]: MIN <= value <= MAX
//!
//! # Example
//!
//! ```rust
//! use stillwater::refined::{Refined, Positive, NonZero, InRange};
//!
//! // Positive values only
//! type PositiveI32 = Refined<i32, Positive>;
//! let n = PositiveI32::new(42).unwrap();
//! assert!(PositiveI32::new(0).is_err());
//! assert!(PositiveI32::new(-5).is_err());
//!
//! // Range validation
//! type Percentage = Refined<i32, InRange<0, 100>>;
//! let pct = Percentage::new(75).unwrap();
//! assert!(Percentage::new(150).is_err());
//! ```

use super::super::Predicate;

/// Value must be positive (> 0)
#[derive(Debug, Clone, Copy, Default)]
pub struct Positive;

/// Value must be non-negative (>= 0)
#[derive(Debug, Clone, Copy, Default)]
pub struct NonNegative;

/// Value must be negative (< 0)
#[derive(Debug, Clone, Copy, Default)]
pub struct Negative;

/// Value must be non-zero (!= 0)
#[derive(Debug, Clone, Copy, Default)]
pub struct NonZero;

/// Value must be in range [MIN, MAX] (inclusive)
#[derive(Debug, Clone, Copy, Default)]
pub struct InRange<const MIN: i64, const MAX: i64>;

// Macro to reduce repetition for signed integer implementations
macro_rules! impl_signed_numeric_predicate {
    ($pred:ty, $check:expr, $msg:expr, $desc:expr, [$($ty:ty),+]) => {
        $(
            impl Predicate<$ty> for $pred {
                type Error = &'static str;

                fn check(value: &$ty) -> Result<(), Self::Error> {
                    if $check(*value) {
                        Ok(())
                    } else {
                        Err($msg)
                    }
                }

                fn description() -> &'static str {
                    $desc
                }
            }
        )+
    };
}

// Positive for signed integers
impl_signed_numeric_predicate!(
    Positive,
    |v| v > 0,
    "value must be positive",
    "positive number (> 0)",
    [i8, i16, i32, i64, i128, isize]
);

// Positive for floats (need separate macro due to comparison)
impl Predicate<f32> for Positive {
    type Error = &'static str;

    fn check(value: &f32) -> Result<(), Self::Error> {
        if *value > 0.0 {
            Ok(())
        } else {
            Err("value must be positive")
        }
    }

    fn description() -> &'static str {
        "positive number (> 0)"
    }
}

impl Predicate<f64> for Positive {
    type Error = &'static str;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if *value > 0.0 {
            Ok(())
        } else {
            Err("value must be positive")
        }
    }

    fn description() -> &'static str {
        "positive number (> 0)"
    }
}

// NonNegative for signed integers
impl_signed_numeric_predicate!(
    NonNegative,
    |v| v >= 0,
    "value must be non-negative",
    "non-negative number (>= 0)",
    [i8, i16, i32, i64, i128, isize]
);

// NonNegative for floats
impl Predicate<f32> for NonNegative {
    type Error = &'static str;

    fn check(value: &f32) -> Result<(), Self::Error> {
        if *value >= 0.0 {
            Ok(())
        } else {
            Err("value must be non-negative")
        }
    }

    fn description() -> &'static str {
        "non-negative number (>= 0)"
    }
}

impl Predicate<f64> for NonNegative {
    type Error = &'static str;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if *value >= 0.0 {
            Ok(())
        } else {
            Err("value must be non-negative")
        }
    }

    fn description() -> &'static str {
        "non-negative number (>= 0)"
    }
}

// Negative for signed integers
impl_signed_numeric_predicate!(
    Negative,
    |v| v < 0,
    "value must be negative",
    "negative number (< 0)",
    [i8, i16, i32, i64, i128, isize]
);

// Negative for floats
impl Predicate<f32> for Negative {
    type Error = &'static str;

    fn check(value: &f32) -> Result<(), Self::Error> {
        if *value < 0.0 {
            Ok(())
        } else {
            Err("value must be negative")
        }
    }

    fn description() -> &'static str {
        "negative number (< 0)"
    }
}

impl Predicate<f64> for Negative {
    type Error = &'static str;

    fn check(value: &f64) -> Result<(), Self::Error> {
        if *value < 0.0 {
            Ok(())
        } else {
            Err("value must be negative")
        }
    }

    fn description() -> &'static str {
        "negative number (< 0)"
    }
}

// NonZero for integers (both signed and unsigned)
macro_rules! impl_nonzero {
    ($($ty:ty),+) => {
        $(
            impl Predicate<$ty> for NonZero {
                type Error = &'static str;

                fn check(value: &$ty) -> Result<(), Self::Error> {
                    if *value != 0 {
                        Ok(())
                    } else {
                        Err("value must be non-zero")
                    }
                }

                fn description() -> &'static str {
                    "non-zero number (!= 0)"
                }
            }
        )+
    };
}

impl_nonzero!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

// InRange for various integer types
macro_rules! impl_in_range {
    ($($ty:ty),+) => {
        $(
            impl<const MIN: i64, const MAX: i64> Predicate<$ty> for InRange<MIN, MAX> {
                type Error = String;

                fn check(value: &$ty) -> Result<(), Self::Error> {
                    let v = *value as i64;
                    if v >= MIN && v <= MAX {
                        Ok(())
                    } else {
                        Err(format!("value {} must be in range [{}, {}]", value, MIN, MAX))
                    }
                }

                fn description() -> &'static str {
                    "value in range [MIN, MAX]"
                }
            }
        )+
    };
}

impl_in_range!(i8, i16, i32, i64, isize, u8, u16, u32);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refined::Refined;

    type PositiveI32 = Refined<i32, Positive>;
    type NonNegativeI32 = Refined<i32, NonNegative>;
    type NegativeI32 = Refined<i32, Negative>;
    type NonZeroI32 = Refined<i32, NonZero>;
    type Percentage = Refined<i32, InRange<0, 100>>;

    #[test]
    fn test_positive_success() {
        assert!(PositiveI32::new(1).is_ok());
        assert!(PositiveI32::new(42).is_ok());
        assert!(PositiveI32::new(i32::MAX).is_ok());
    }

    #[test]
    fn test_positive_failure() {
        assert!(PositiveI32::new(0).is_err());
        assert!(PositiveI32::new(-1).is_err());
        assert!(PositiveI32::new(i32::MIN).is_err());
    }

    #[test]
    fn test_positive_f64() {
        type PositiveF64 = Refined<f64, Positive>;
        assert!(PositiveF64::new(0.1).is_ok());
        assert!(PositiveF64::new(0.0).is_err());
        assert!(PositiveF64::new(-0.1).is_err());
    }

    #[test]
    fn test_non_negative_success() {
        assert!(NonNegativeI32::new(0).is_ok());
        assert!(NonNegativeI32::new(1).is_ok());
        assert!(NonNegativeI32::new(42).is_ok());
    }

    #[test]
    fn test_non_negative_failure() {
        assert!(NonNegativeI32::new(-1).is_err());
        assert!(NonNegativeI32::new(i32::MIN).is_err());
    }

    #[test]
    fn test_negative_success() {
        assert!(NegativeI32::new(-1).is_ok());
        assert!(NegativeI32::new(-42).is_ok());
        assert!(NegativeI32::new(i32::MIN).is_ok());
    }

    #[test]
    fn test_negative_failure() {
        assert!(NegativeI32::new(0).is_err());
        assert!(NegativeI32::new(1).is_err());
    }

    #[test]
    fn test_non_zero_success() {
        assert!(NonZeroI32::new(1).is_ok());
        assert!(NonZeroI32::new(-1).is_ok());
        assert!(NonZeroI32::new(42).is_ok());
    }

    #[test]
    fn test_non_zero_failure() {
        assert!(NonZeroI32::new(0).is_err());
    }

    #[test]
    fn test_non_zero_unsigned() {
        type NonZeroU32 = Refined<u32, NonZero>;
        assert!(NonZeroU32::new(1).is_ok());
        assert!(NonZeroU32::new(0).is_err());
    }

    #[test]
    fn test_in_range_success() {
        assert!(Percentage::new(0).is_ok());
        assert!(Percentage::new(50).is_ok());
        assert!(Percentage::new(100).is_ok());
    }

    #[test]
    fn test_in_range_failure() {
        assert!(Percentage::new(-1).is_err());
        assert!(Percentage::new(101).is_err());
    }

    #[test]
    fn test_in_range_port() {
        type Port = Refined<u16, InRange<1, 65535>>;
        assert!(Port::new(80).is_ok());
        assert!(Port::new(443).is_ok());
        assert!(Port::new(1).is_ok());
        assert!(Port::new(65535).is_ok());
        assert!(Port::new(0).is_err());
    }

    #[test]
    fn test_description() {
        assert_eq!(
            <Positive as Predicate<i32>>::description(),
            "positive number (> 0)"
        );
        assert_eq!(
            <NonNegative as Predicate<i32>>::description(),
            "non-negative number (>= 0)"
        );
        assert_eq!(
            <Negative as Predicate<i32>>::description(),
            "negative number (< 0)"
        );
        assert_eq!(
            <NonZero as Predicate<i32>>::description(),
            "non-zero number (!= 0)"
        );
    }
}
