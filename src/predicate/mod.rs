//! Predicate combinators for composable validation logic
//!
//! This module provides composable predicate combinators for use in validation pipelines.
//! Predicates can be combined using logical operators (`and`, `or`, `not`) to build
//! complex validation rules from simple, reusable pieces.
//!
//! # Philosophy
//!
//! Instead of writing verbose boolean expressions or ad-hoc helper functions,
//! predicate combinators allow you to:
//!
//! - Build complex predicates from simple, reusable pieces
//! - Compose predicates using familiar logical operators
//! - Integrate seamlessly with `Validation` for error accumulation
//!
//! # Example
//!
//! ```rust
//! use stillwater::predicate::*;
//!
//! // Define reusable predicates for String type
//! let valid_len = len_between(3, 20);
//! let chars_ok = all_chars(|c: char| c.is_alphanumeric() || c == '_');
//!
//! // Check individual predicates
//! assert!(valid_len.check(&String::from("john_doe")));
//! assert!(!valid_len.check(&String::from("ab"))); // too short
//! assert!(!chars_ok.check(&String::from("invalid-name"))); // contains hyphen
//! ```
//!
//! # Integration with Validation
//!
//! ```rust
//! use stillwater::{Validation, predicate::*};
//!
//! let result = validate(String::from("hello"), len_min(3), "too short");
//! assert_eq!(result, Validation::success(String::from("hello")));
//!
//! let result = Validation::success(String::from("hello"))
//!     .ensure(len_min(3), "too short")
//!     .ensure(len_max(10), "too long");
//! assert_eq!(result, Validation::success(String::from("hello")));
//! ```

mod collection;
mod combinators;
mod number;
mod string;
mod validation;

pub mod prelude;

// Re-export core trait
pub use combinators::{Predicate, PredicateExt};

// Re-export combinator types
pub use combinators::{all_of, any_of, none_of, AllOf, And, AnyOf, NoneOf, Not, Or};

// Re-export string predicates
pub use string::{
    all_chars, any_char, contains, ends_with, is_alphabetic, is_alphanumeric, is_ascii, is_numeric,
    len_between, len_eq, len_max, len_min, not_empty, starts_with, AllChars, AnyChar, Contains,
    EndsWith, LenBetween, NotEmpty, StartsWith,
};

// Re-export number predicates
pub use number::{
    between, eq, ge, gt, le, lt, ne, negative, non_negative, positive, Between, Eq, Ge, Gt, Le, Lt,
    Ne,
};

// Re-export collection predicates
pub use collection::{
    all, any, contains_element, has_len, has_max_len, has_min_len, is_empty, is_not_empty, All,
    Any, ContainsElement, HasLen, HasMaxLen, HasMinLen, IsEmpty, IsNotEmpty,
};

// Re-export validation integration
pub use validation::{validate, validate_with};
