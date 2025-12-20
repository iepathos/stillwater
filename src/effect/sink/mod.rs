//! Sink Effect for streaming output with constant memory.
//!
//! The Sink Effect enables streaming logs, metrics, or events during computation
//! without accumulating them in memory, unlike the Writer Effect which collects
//! all writes until execution completes.
//!
//! # Overview
//!
//! Where Writer Effect accumulates all output:
//!
//! ```rust,ignore
//! // Writer Effect - O(n) memory, all logs collected
//! let (result, logs) = traverse_writer(million_items, process)
//!     .run_writer(&env).await;
//! // logs now contains 1M entries in memory
//! ```
//!
//! Sink Effect streams immediately with constant memory:
//!
//! ```rust
//! use stillwater::effect::sink::prelude::*;
//!
//! # tokio_test::block_on(async {
//! let items = vec![1, 2, 3];
//! let effect = traverse_sink(items, |n| {
//!     emit::<_, String, ()>(format!("Processing: {}", n))
//!         .map(move |_| n * 10)
//! });
//!
//! // Stream to console - O(1) memory regardless of item count
//! let result = effect.run_with_sink(&(), |log| async move {
//!     println!("{}", log);
//! }).await;
//!
//! assert_eq!(result, Ok(vec![10, 20, 30]));
//! # });
//! ```
//!
//! # Key Features
//!
//! - **Constant memory**: Items streamed immediately, not accumulated
//! - **Real-time output**: Logs appear as they happen, not after completion
//! - **Flexible sinks**: Write to stdout, files, channels, databases
//! - **Testable**: `run_collecting` bridges to Writer semantics for assertions
//! - **Async sinks**: Support for async I/O operations
//!
//! # When to Use
//!
//! | Trait | Purpose | Memory | Best For |
//! |-------|---------|--------|----------|
//! | WriterEffect | Pure accumulation | O(n) | Testing, short chains, audit trails |
//! | SinkEffect | Streaming output | O(1) | Production, high-volume, real-time logs |
//!
//! # Module Structure
//!
//! - [`SinkEffect`] - Core trait extending Effect with streaming
//! - [`SinkEffectExt`] - Extension trait providing combinator methods
//! - [`emit()`], [`emit_many`] - Functions to emit items
//! - [`into_sink()`] - Lift regular Effects into SinkEffect
//!
//! # Example: Testing vs Production
//!
//! ```rust
//! use stillwater::effect::sink::prelude::*;
//!
//! # tokio_test::block_on(async {
//! let effect = emit::<_, String, ()>("step 1".to_string())
//!     .and_then(|_| emit("step 2".to_string()))
//!     .and_then(|_| emit("step 3".to_string()))
//!     .map(|_| "done");
//!
//! // Testing: collect for assertions
//! let (result, logs) = effect.run_collecting(&()).await;
//! assert_eq!(result, Ok("done"));
//! assert_eq!(logs, vec!["step 1", "step 2", "step 3"]);
//! # });
//! ```

mod and_then;
mod boxed;
mod combinators;
mod emit;
mod ext;
mod into_sink;
mod map;
mod map_err;
mod or_else;
pub mod prelude;
mod tap_emit;
mod trait_def;
mod zip;

// Re-export core trait
pub use trait_def::SinkEffect;

// Re-export extension trait
pub use ext::SinkEffectExt;

// Re-export constructors
pub use emit::{emit, emit_many, Emit, EmitMany};

// Re-export lifting function
pub use into_sink::{into_sink, IntoSink};

// Re-export combinator types
pub use and_then::SinkAndThen;
pub use map::SinkMap;
pub use map_err::SinkMapErr;
pub use or_else::SinkOrElse;
pub use tap_emit::TapEmit;
pub use zip::SinkZip;

// Re-export boxed types
pub use boxed::BoxedSinkEffect;

// Re-export collection combinators
pub use combinators::{fold_sink, traverse_sink, FoldSink, TraverseSink};

#[cfg(test)]
mod tests;
