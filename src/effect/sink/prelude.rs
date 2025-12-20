//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and functions
//! from the sink module, allowing users to quickly get started
//! with a single `use` statement.
//!
//! # Recommended Usage
//!
//! ```rust
//! use stillwater::effect::sink::prelude::*;
//!
//! # tokio_test::block_on(async {
//! let effect = emit::<_, String, ()>("Starting".to_string())
//!     .map(|_| 42)
//!     .tap_emit(|n| format!("Result: {}", n));
//!
//! let (result, logs) = effect.run_collecting(&()).await;
//! assert_eq!(result, Ok(42));
//! assert_eq!(logs, vec!["Starting".to_string(), "Result: 42".to_string()]);
//! # });
//! ```

// Core traits
pub use crate::effect::sink::ext::SinkEffectExt;
pub use crate::effect::sink::trait_def::SinkEffect;

// Constructors
pub use crate::effect::sink::emit::{emit, emit_many, Emit, EmitMany};
pub use crate::effect::sink::into_sink::{into_sink, IntoSink};

// Combinator types
pub use crate::effect::sink::and_then::SinkAndThen;
pub use crate::effect::sink::map::SinkMap;
pub use crate::effect::sink::map_err::SinkMapErr;
pub use crate::effect::sink::or_else::SinkOrElse;
pub use crate::effect::sink::tap_emit::TapEmit;
pub use crate::effect::sink::zip::SinkZip;

// Boxed type
pub use crate::effect::sink::boxed::BoxedSinkEffect;

// Collection combinators
pub use crate::effect::sink::combinators::{fold_sink, traverse_sink, FoldSink, TraverseSink};
