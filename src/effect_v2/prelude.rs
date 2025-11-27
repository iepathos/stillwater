//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and functions
//! from the effect_v2 module, allowing users to quickly get started
//! with a single `use` statement.
//!
//! # Example
//!
//! ```rust,ignore
//! use stillwater::effect_v2::prelude::*;
//!
//! let effect = pure::<_, String, ()>(42)
//!     .map(|x| x * 2)
//!     .and_then(|x| pure(x + 1));
//!
//! let result = effect.execute(&()).await;
//! assert_eq!(result, Ok(85));
//! ```

// Traits
pub use crate::effect_v2::ext::EffectExt;
pub use crate::effect_v2::trait_def::Effect;

// Boxed Effect
pub use crate::effect_v2::boxed::{BoxFuture, BoxedEffect, BoxedLocalEffect};

// Combinator Types (for advanced use, usually `impl Effect` suffices)
pub use crate::effect_v2::combinators::{
    AndThen, Fail, FromAsync, FromFn, FromResult, Map, MapErr, OrElse, Pure,
};

// Reader Types
pub use crate::effect_v2::reader::{Ask, Asks, Local};

// Bracket
pub use crate::effect_v2::bracket::Bracket;

// Constructors
pub use crate::effect_v2::constructors::{
    ask, asks, fail, from_async, from_fn, from_option, from_result, local, pure,
};

// Bracket constructor
pub use crate::effect_v2::bracket::{bracket, bracket_simple};

// Parallel (homogeneous, requires boxing)
pub use crate::effect_v2::parallel::{par_all, par_try_all, race};

// Parallel (heterogeneous, zero-cost)
pub use crate::effect_v2::parallel::{par2, par3, par4};

// Re-export the par! macro
pub use crate::par;
