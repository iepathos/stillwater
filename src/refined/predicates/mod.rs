//! Predefined predicates for common refinement patterns
//!
//! This module provides ready-to-use predicates for:
//! - **Numeric constraints**: [`numeric::Positive`], [`numeric::NonNegative`], [`numeric::Negative`], [`numeric::NonZero`], [`numeric::InRange`]
//! - **String constraints**: [`string::NonEmpty`], [`string::Trimmed`], [`string::MaxLength`], [`string::MinLength`]
//! - **Collection constraints**: [`collection::MaxSize`], [`collection::MinSize`] (for `Vec<T>`)
//!
//! # Example
//!
//! ```rust
//! use stillwater::refined::{Refined, Positive, NonEmpty, InRange};
//!
//! // Positive integers
//! type PositiveI32 = Refined<i32, Positive>;
//! let age = PositiveI32::new(25).unwrap();
//!
//! // Non-empty strings
//! type NonEmptyString = Refined<String, NonEmpty>;
//! let name = NonEmptyString::new("Alice".to_string()).unwrap();
//!
//! // Range-constrained values
//! type Percentage = Refined<i32, InRange<0, 100>>;
//! let pct = Percentage::new(75).unwrap();
//! ```

pub mod collection;
pub mod numeric;
pub mod string;
