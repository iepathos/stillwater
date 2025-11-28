//! Zero-cost Effect trait with opt-in boxing.
//!
//! This module provides a redesigned Effect system that is **zero-cost by default**
//! with **opt-in boxing** when type erasure is needed, following the established
//! `futures` crate pattern.
//!
//! # Key Differences from Old API
//!
//! | Old API | New API |
//! |---------|---------|
//! | `Effect<T, E, Env>` struct (boxed per combinator) | `impl Effect<Output=T, Error=E, Env=Env>` (zero-cost) |
//! | `Effect::pure(x)` | `pure::<_, E, Env>(x)` |
//! | `Effect::fail(e)` | `fail::<T, _, Env>(e)` |
//! | Automatic type erasure | Explicit `.boxed()` when needed |
//!
//! # Zero-Cost by Default
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//!
//! // Zero heap allocations - compiler can inline everything
//! let effect = pure::<_, String, ()>(42)
//!     .map(|x| x + 1)           // Returns Map<Pure<...>, ...>
//!     .and_then(|x| pure(x * 2)) // Returns AndThen<Map<...>, ...>
//!     .map(|x| x.to_string());   // Returns Map<AndThen<...>, ...>
//!
//! // Type: Map<AndThen<Map<Pure<i32, String, ()>, ...>, ...>, ...>
//! // NO heap allocation!
//! ```
//!
//! # When to Use Boxing
//!
//! Boxing is needed in exactly three cases:
//!
//! ## 1. Storing in Collections
//!
//! ```rust,ignore
//! // Can't put different types in a Vec!
//! let effects: Vec<BoxedEffect<i32, E, Env>> = vec![
//!     pure(1).boxed(),
//!     pure(2).map(|x| x * 2).boxed(),
//! ];
//! ```
//!
//! ## 2. Recursive Effects
//!
//! ```rust,ignore
//! fn countdown(n: i32) -> BoxedEffect<i32, String, ()> {
//!     if n <= 0 {
//!         pure(0).boxed()
//!     } else {
//!         pure(n)
//!             .and_then(move |x| countdown(x - 1).map(move |sum| x + sum))
//!             .boxed()
//!     }
//! }
//! ```
//!
//! ## 3. Match Arms with Different Effect Types
//!
//! ```rust,ignore
//! fn get_user(source: DataSource) -> BoxedEffect<User, E, Env> {
//!     match source {
//!         DataSource::Cache => pure(user).boxed(),
//!         DataSource::Database => fetch_from_db().boxed(),
//!     }
//! }
//! ```
//!
//! # Environment Cloning
//!
//! The environment (`Env`) must implement `Clone`. This is required for boxing,
//! where the environment is cloned into the boxed future to achieve `'static`
//! lifetime. This is typically cheap when environments contain `Arc`-wrapped
//! resources:
//!
//! ```rust,ignore
//! #[derive(Clone)]
//! struct AppEnv {
//!     db: Arc<DatabasePool>,      // Clone is cheap (Arc refcount)
//!     config: Arc<Config>,
//!     http: Arc<HttpClient>,
//! }
//! ```

pub mod boxed;
pub mod bracket;
pub mod combinators;
pub mod compat;
pub mod constructors;
pub mod context;
pub mod ext;
pub mod parallel;
pub mod prelude;
pub mod reader;
#[cfg(feature = "async")]
pub mod retry;
#[cfg(feature = "tracing")]
pub mod tracing;
mod trait_def;

// Re-export core trait
pub use trait_def::Effect;

// Re-export extension trait
pub use ext::EffectExt;

// Re-export boxed types
pub use boxed::{BoxFuture, BoxedEffect, BoxedLocalEffect};

// Re-export all combinator types
pub use combinators::{
    AndThen, AndThenAuto, AndThenRef, Check, Fail, FromAsync, FromFn, FromResult, Map, MapErr,
    OrElse, Pure, Tap, With, Zip, Zip3, Zip4, Zip5, Zip6, Zip7, Zip8, ZipWith,
};

// Re-export reader types
pub use reader::{Ask, Asks, Local};

// Re-export bracket
pub use bracket::{bracket, bracket_simple, Bracket};

// Re-export constructors
pub use constructors::{
    ask, asks, fail, from_async, from_fn, from_option, from_result, from_validation, local, pure,
    zip3, zip4, zip5, zip6, zip7, zip8,
};

// Re-export parallel functions
pub use parallel::{par2, par3, par4, par_all, par_all_limit, par_try_all, race};

// Re-export context trait
pub use context::{EffectContext, EffectContextChain};

// Re-export retry functions (when async feature is enabled)
#[cfg(feature = "async")]
pub use retry::{retry, retry_if, retry_with_hooks, with_timeout};

// Re-export tracing (when tracing feature is enabled)
#[cfg(feature = "tracing")]
pub use tracing::{EffectTracingExt, Instrument};

// Re-export compatibility items
#[allow(deprecated)]
pub use compat::{LegacyConstructors, LegacyEffect, RunStandalone};

#[cfg(test)]
mod tests;
