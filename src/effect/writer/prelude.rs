//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and functions
//! from the writer module, allowing users to quickly get started
//! with a single `use` statement.
//!
//! # Recommended Usage
//!
//! ```rust
//! use stillwater::effect::writer::prelude::*;
//! use stillwater::effect::prelude::*;
//!
//! # tokio_test::block_on(async {
//! let effect = tell_one::<_, String, ()>("Starting".to_string())
//!     .map(|_| 42)
//!     .tap_tell(|n| vec![format!("Result: {}", n)]);
//!
//! let (result, logs) = effect.run_writer(&()).await;
//! assert_eq!(result, Ok(42));
//! assert_eq!(logs, vec!["Starting".to_string(), "Result: 42".to_string()]);
//! # });
//! ```

// Core traits
pub use crate::effect::writer::ext::WriterEffectExt;
pub use crate::effect::writer::trait_def::WriterEffect;

// Constructors
pub use crate::effect::writer::into_writer::{into_writer, IntoWriter};
pub use crate::effect::writer::tell::{tell, tell_one, Tell};

// Combinator types
pub use crate::effect::writer::and_then::WriterAndThen;
pub use crate::effect::writer::censor::Censor;
pub use crate::effect::writer::listen::Listen;
pub use crate::effect::writer::map::WriterMap;
pub use crate::effect::writer::map_err::WriterMapErr;
pub use crate::effect::writer::or_else::WriterOrElse;
pub use crate::effect::writer::pass::Pass;
pub use crate::effect::writer::tap_tell::TapTell;
pub use crate::effect::writer::zip::WriterZip;

// Boxed type
pub use crate::effect::writer::boxed::BoxedWriterEffect;

// Collection combinators
pub use crate::effect::writer::combinators::{fold_writer, traverse_writer};
