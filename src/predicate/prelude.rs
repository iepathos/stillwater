//! Predicate prelude for convenient imports
//!
//! This module re-exports the most commonly used predicate types and functions.
//!
//! # Example
//!
//! ```rust
//! use stillwater::predicate::prelude::*;
//!
//! let valid_age = ge(0).and(le(150));
//! assert!(valid_age.check(&25));
//! ```

// Core trait
pub use super::combinators::{Predicate, PredicateExt};

// Logical combinators
pub use super::combinators::{all_of, any_of, none_of, And, Not, Or};

// String predicates
pub use super::string::{
    all_chars, any_char, contains, ends_with, is_alphabetic, is_alphanumeric, is_ascii, is_numeric,
    len_between, len_eq, len_max, len_min, not_empty, starts_with,
};

// Number predicates
pub use super::number::{between, eq, ge, gt, le, lt, ne, negative, non_negative, positive};

// Collection predicates
pub use super::collection::{
    all, any, contains_element, has_len, has_max_len, has_min_len, is_empty, is_not_empty,
};

// Validation integration
pub use super::validation::{validate, validate_with};
