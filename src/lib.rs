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
//! use stillwater::{Validation, Effect};
//!
//! // Accumulate all validation errors (once Validation is implemented)
//! let user = Validation::all((
//!     validate_email(input),
//!     validate_age(input),
//!     validate_name(input),
//! ))?;
//!
//! // Compose effects with pure logic (once Effect is implemented)
//! fn create_user(input: UserInput) -> Effect<User, Error, AppEnv> {
//!     Effect::from_validation(validate_user(input))
//!         .and_then(|user| save_user(user))
//!         .context("Creating user")
//! }
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod context;
pub mod effect;
pub mod semigroup;
pub mod validation;

// Re-exports
pub use context::ContextError;
pub use effect::{Effect, EffectContext};
pub use semigroup::Semigroup;
pub use validation::Validation;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::context::ContextError;
    pub use crate::effect::{Effect, EffectContext};
    pub use crate::semigroup::Semigroup;
    pub use crate::validation::Validation;
}
