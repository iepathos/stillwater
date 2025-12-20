//! Writer Effect for accumulating values alongside computation.
//!
//! The Writer Effect enables accumulating logs, metrics, or audit trails
//! alongside computation without threading state through every function.
//!
//! # Overview
//!
//! Instead of manually threading an accumulator through every function:
//!
//! ```rust,ignore
//! fn process(x: i32, logs: &mut Vec<String>) -> Result<i32, Error> {
//!     logs.push("Starting".into());
//!     let y = step1(x, logs)?;
//!     logs.push(format!("Step 1: {}", y));
//!     Ok(y)
//! }
//! ```
//!
//! Use the Writer Effect for automatic accumulation:
//!
//! ```rust
//! use stillwater::effect::writer::prelude::*;
//! use stillwater::effect::prelude::*;
//!
//! # tokio_test::block_on(async {
//! let effect = tell_one::<_, String, ()>("Starting".to_string())
//!     .and_then(|_| into_writer::<_, _, Vec<String>>(pure::<_, String, ()>(42)))
//!     .tap_tell(|y| vec![format!("Step 1: {}", y)]);
//!
//! let (result, logs) = effect.run_writer(&()).await;
//! assert_eq!(result, Ok(42));
//! assert_eq!(logs, vec!["Starting".to_string(), "Step 1: 42".to_string()]);
//! # });
//! ```
//!
//! # Key Features
//!
//! - **Monoid-based accumulation**: Works with any `W: Monoid`
//! - **Type-safe log types**: Different effects can use different accumulator types
//! - **Zero-cost abstractions**: Concrete types, no boxing for Writer infrastructure
//! - **Composable with Effect**: Full integration with existing combinators
//!
//! # Module Structure
//!
//! - [`WriterEffect`] - Core trait extending Effect with accumulation
//! - [`WriterEffectExt`] - Extension trait providing combinator methods
//! - [`tell()`], [`tell_one`] - Functions to emit values
//! - [`into_writer()`] - Lift regular Effects into WriterEffect
//!
//! # Example: Audit Logging
//!
//! ```rust
//! use stillwater::effect::writer::prelude::*;
//! use stillwater::effect::prelude::*;
//!
//! #[derive(Debug, Clone, PartialEq)]
//! enum AuditEvent {
//!     Started,
//!     Completed(i32),
//! }
//!
//! # tokio_test::block_on(async {
//! let effect = tell_one::<_, String, ()>(AuditEvent::Started)
//!     .and_then(|_| into_writer::<_, _, Vec<AuditEvent>>(pure::<_, String, ()>(42)))
//!     .tap_tell(|n| vec![AuditEvent::Completed(*n)]);
//!
//! let (result, events) = effect.run_writer(&()).await;
//! assert_eq!(result, Ok(42));
//! assert_eq!(events, vec![AuditEvent::Started, AuditEvent::Completed(42)]);
//! # });
//! ```

mod and_then;
mod boxed;
mod censor;
mod combinators;
mod ext;
mod into_writer;
mod listen;
mod map;
mod map_err;
mod or_else;
mod pass;
pub mod prelude;
mod tap_tell;
mod tell;
mod trait_def;
mod zip;

// Re-export core trait
pub use trait_def::WriterEffect;

// Re-export extension trait
pub use ext::WriterEffectExt;

// Re-export constructors
pub use tell::{tell, tell_one};

// Re-export lifting function
pub use into_writer::{into_writer, IntoWriter};

// Re-export combinator types
pub use and_then::WriterAndThen;
pub use censor::Censor;
pub use listen::Listen;
pub use map::WriterMap;
pub use map_err::WriterMapErr;
pub use or_else::WriterOrElse;
pub use pass::Pass;
pub use tap_tell::TapTell;
pub use tell::Tell;
pub use zip::WriterZip;

// Re-export boxed types
pub use boxed::BoxedWriterEffect;

// Re-export collection combinators
pub use combinators::{fold_writer, traverse_writer};

#[cfg(test)]
mod tests;
