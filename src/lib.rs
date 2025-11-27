#![cfg_attr(feature = "try_trait", feature(try_trait_v2))]
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
//! ## Effect System
//!
//! The effect system has been redesigned to be **zero-cost by default** with **opt-in boxing**
//! when type erasure is needed, following the established `futures` crate pattern.
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//!
//! // Zero heap allocations - compiler can inline everything
//! let effect = pure::<_, String, ()>(42)
//!     .map(|x| x + 1)
//!     .and_then(|x| pure(x * 2));
//!
//! // Use .boxed() when you need type erasure
//! let effects: Vec<BoxedEffect<i32, String, ()>> = vec![
//!     pure(1).boxed(),
//!     pure(2).map(|x| x * 2).boxed(),
//! ];
//! ```
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
pub mod retry;
pub mod semigroup;
pub mod testing;
pub mod traverse;
pub mod validation;

// Re-exports - Effect system (zero-cost by default)
pub use effect::{BoxedEffect, Effect, EffectContext, EffectContextChain, EffectExt};

// Re-export boxed types
pub use effect::boxed::{BoxFuture, BoxedLocalEffect};

// Re-export constructors
pub use effect::constructors::{
    ask, asks, fail, from_async, from_fn, from_option, from_result, from_validation, local, pure,
};

// Re-export parallel functions
pub use effect::parallel::{par2, par3, par4, par_all, par_all_limit, par_try_all, race};

// Re-export combinator types (for advanced use)
pub use effect::combinators::{
    AndThen, AndThenAuto, AndThenRef, Check, Fail, FromAsync, FromFn, FromResult, Map, MapErr,
    OrElse, Pure, Tap, With,
};

// Re-export reader types
pub use effect::reader::{Ask, Asks, Local};

// Re-export bracket
pub use effect::bracket::{bracket, bracket_simple, Bracket};

// Re-export compat items
#[allow(deprecated)]
pub use effect::compat::{LegacyConstructors, LegacyEffect, RunStandalone};

// Re-export tracing (when feature enabled)
#[cfg(feature = "tracing")]
pub use effect::tracing::{EffectTracingExt, Instrument};

// Other re-exports
pub use context::ContextError;
pub use io::IO;
pub use monoid::Monoid;
pub use nonempty::NonEmptyVec;
pub use retry::{
    JitterStrategy, RetryEvent, RetryExhausted, RetryPolicy, RetryStrategy, TimeoutError,
};
pub use semigroup::{First, Intersection, Last, Semigroup};
pub use validation::Validation;

/// Prelude module for convenient imports
pub mod prelude {
    // Effect system
    pub use crate::effect::prelude::*;

    // Other types
    pub use crate::context::ContextError;
    pub use crate::io::IO;
    pub use crate::monoid::Monoid;
    pub use crate::nonempty::NonEmptyVec;
    pub use crate::retry::{RetryEvent, RetryExhausted, RetryPolicy, TimeoutError};
    pub use crate::semigroup::{First, Intersection, Last, Semigroup};
    pub use crate::testing::{MockEnv, TestEffect};
    pub use crate::traverse::{sequence, sequence_effect, traverse, traverse_effect};
    pub use crate::validation::Validation;
    pub use crate::{assert_failure, assert_success, assert_validation_errors};
}
