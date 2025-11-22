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
//! ```rust,ignore
//! use stillwater::{Validation, Effect, IO};
//!
//! // Accumulate all validation errors
//! let user = Validation::all((
//!     validate_email(input),
//!     validate_age(input),
//!     validate_name(input),
//! ))?;
//!
//! // Compose effects with pure logic
//! fn create_user(input: UserInput) -> Effect<User, Error, AppEnv> {
//!     Effect::from_validation(validate_user(input))
//!         .and_then(|user| IO::execute(|db| db.save(&user)))
//!         .context("Creating user")
//! }
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod validation;
pub mod effect;
pub mod context;
pub mod semigroup;

// Re-exports
pub use validation::Validation;
pub use effect::Effect;
pub use context::ContextError;
pub use semigroup::Semigroup;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::validation::Validation;
    pub use crate::effect::Effect;
    pub use crate::context::ContextError;
    pub use crate::semigroup::Semigroup;
}
