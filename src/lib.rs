//! # Stillwater
//!
//! > *"Still waters run pure"*
//!
//! A Rust library for pragmatic effect composition and validation.
//!
//! ## Philosophy
//!
//! **Stillwater** embodies the principle of **pure core, imperative shell**:
//! - **Still** = Pure functions (unchanging, referentially transparent)
//! - **Water** = Effects (flowing, performing I/O)
//!
//! ## Quick Example
//!
//! ```rust
//! use stillwater::Validation;
//!
//! // Accumulate all validation errors
//! fn validate_email(email: &str) -> Validation<String, Vec<String>> {
//!     if email.contains('@') {
//!         Validation::success(email.to_string())
//!     } else {
//!         Validation::failure(vec!["Email must contain @".to_string()])
//!     }
//! }
//!
//! fn validate_age(age: i32) -> Validation<i32, Vec<String>> {
//!     if age >= 18 {
//!         Validation::success(age)
//!     } else {
//!         Validation::failure(vec!["Must be 18 or older".to_string()])
//!     }
//! }
//!
//! // Collect all errors at once
//! let result = Validation::<(String, i32), Vec<String>>::all((
//!     validate_email("user@example.com"),
//!     validate_age(25),
//! ));
//!
//! match result {
//!     Validation::Success((email, age)) => {
//!         println!("Valid: {} is {} years old", email, age);
//!     }
//!     Validation::Failure(errors) => {
//!         println!("Errors: {:?}", errors);
//!     }
//! }
//! ```
//!
//! For more examples, see the [examples](https://github.com/iepathos/stillwater/tree/master/examples) directory.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod context;
pub mod effect;
pub mod io;
pub mod monoid;
pub mod nonempty;
pub mod semigroup;
pub mod validation;

// Re-exports
pub use context::ContextError;
pub use effect::{Effect, EffectContext};
pub use io::IO;
pub use monoid::Monoid;
pub use nonempty::NonEmptyVec;
pub use semigroup::{First, Intersection, Last, Semigroup};
pub use validation::Validation;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::context::ContextError;
    pub use crate::effect::{Effect, EffectContext};
    pub use crate::io::IO;
    pub use crate::monoid::Monoid;
    pub use crate::nonempty::NonEmptyVec;
    pub use crate::semigroup::{First, Intersection, Last, Semigroup};
    pub use crate::validation::Validation;
}
