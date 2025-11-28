//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and functions
//! from the effect module, allowing users to quickly get started
//! with a single `use` statement.
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//!
//! let effect = pure::<_, String, ()>(42)
//!     .map(|x| x * 2)
//!     .and_then(|x| pure(x + 1));
//!
//! let result = effect.execute(&()).await;
//! assert_eq!(result, Ok(85));
//! ```

// Traits
pub use crate::effect::context::{EffectContext, EffectContextChain};
pub use crate::effect::ext::EffectExt;
pub use crate::effect::trait_def::Effect;

// Boxed Effect
pub use crate::effect::boxed::{BoxFuture, BoxedEffect, BoxedLocalEffect};

// Combinator Types (for advanced use, usually `impl Effect` suffices)
pub use crate::effect::combinators::{
    AndThen, AndThenAuto, AndThenRef, Check, Fail, FromAsync, FromFn, FromResult, Map, MapErr,
    OrElse, Pure, Tap, With,
};

// Reader Types
pub use crate::effect::reader::{Ask, Asks, Local};

// Bracket types and constructors
#[allow(deprecated)]
pub use crate::effect::bracket::bracket_simple;
pub use crate::effect::bracket::{
    acquiring, bracket, bracket2, bracket3, bracket_full, bracket_sync, Acquiring, Bracket,
    Bracket2, Bracket3, BracketError, BracketFull, BracketSync, Resource, ResourceWith,
};

// Constructors
pub use crate::effect::constructors::{
    ask, asks, fail, from_async, from_fn, from_option, from_result, from_validation, local, pure,
};

// Parallel (homogeneous, requires boxing)
pub use crate::effect::parallel::{par_all, par_all_limit, par_try_all, race};

// Parallel (heterogeneous, zero-cost)
pub use crate::effect::parallel::{par2, par3, par4};

// Re-export the par! macro
pub use crate::par;

// Retry functions (when async feature is enabled)
#[cfg(feature = "async")]
pub use crate::effect::retry::{retry, retry_if, retry_with_hooks, with_timeout};

// Tracing (when tracing feature is enabled)
#[cfg(feature = "tracing")]
pub use crate::effect::tracing::EffectTracingExt;

// Compat traits for running effects
pub use crate::effect::compat::RunStandalone;
