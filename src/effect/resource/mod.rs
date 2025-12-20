//! Compile-time resource tracking for effects.
//!
//! This module provides optional resource tracking that lifts resource safety
//! to the type level. It enables:
//!
//! - **Type-level documentation** of resource acquisition/release
//! - **Compile-time detection** of resource leaks
//! - **Protocol enforcement** (e.g., transaction begin/end)
//! - **Zero runtime overhead** (purely type-level)
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use stillwater::effect::prelude::*;
//! use stillwater::effect::resource::*;
//!
//! // Mark an effect as acquiring a file resource
//! fn open_file(path: &str) -> impl ResourceEffect<Acquires = Has<FileRes>> {
//!     pure(FileHandle::new(path)).acquires::<FileRes>()
//! }
//!
//! // Mark an effect as releasing a file resource
//! fn close_file(handle: FileHandle) -> impl ResourceEffect<Releases = Has<FileRes>> {
//!     pure(()).releases::<FileRes>()
//! }
//!
//! // Use the bracket builder for guaranteed resource safety (ergonomic syntax)
//! fn read_file_safe(path: &str) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
//!     bracket::<FileRes>()
//!         .acquire(open_file(path))
//!         .release(|h| async move { close_file(h).run(&()).await })
//!         .use_fn(|h| read_contents(h))
//! }
//! ```
//!
//! # Module Structure
//!
//! - [`markers`] - Resource kind markers (`FileRes`, `DbRes`, etc.)
//! - [`sets`] - Type-level resource sets (`Empty`, `Has<R>`)
//! - [`tracked`] - `Tracked` wrapper and `ResourceEffect` trait
//! - [`ext`] - Extension methods (`.acquires()`, `.releases()`)
//! - [`bracket`] - Resource-safe bracket pattern (`resource_bracket` function)
//! - [`builder`] - Ergonomic builder API (`bracket::<R>()`, `Bracket::<R>::new()`)
//! - [`combinators`] - ResourceEffect implementations for core combinators
//!
//! # Resource Kinds
//!
//! Resource kinds are zero-sized marker types that identify resource types:
//!
//! | Marker | Description |
//! |--------|-------------|
//! | `FileRes` | File handles |
//! | `DbRes` | Database connections |
//! | `LockRes` | Locks/mutexes |
//! | `TxRes` | Transactions |
//! | `SocketRes` | Network sockets |
//!
//! You can define custom resource kinds:
//!
//! ```rust,ignore
//! pub struct MyPoolRes;
//! impl ResourceKind for MyPoolRes {
//!     const NAME: &'static str = "ConnectionPool";
//! }
//! ```
//!
//! # Resource Sets
//!
//! Resources are tracked using type-level sets:
//!
//! - `Empty` - No resources
//! - `Has<R>` - Single resource R
//! - `Has<R, Has<S>>` - Resources R and S
//!
//! # Tracking Mechanisms
//!
//! ## Extension Methods
//!
//! The easiest way to add tracking:
//!
//! ```rust,ignore
//! let effect = some_effect
//!     .acquires::<FileRes>()    // Mark acquisition
//!     .map(|x| process(x))      // Tracking preserved through map
//!     .releases::<FileRes>();   // Mark release
//! ```
//!
//! ## Resource Bracket
//!
//! For guaranteed resource safety, use the builder pattern:
//!
//! ```rust,ignore
//! // Ergonomic builder (recommended)
//! let safe = bracket::<FileRes>()
//!     .acquire(acquire_effect)
//!     .release(|resource| async move { release(resource).await })
//!     .use_fn(|resource| use_resource(resource));
//!
//! // Or with the function (10 type parameters, 9 inferred)
//! let safe = resource_bracket::<FileRes, _, _, _, _, _, _, _, _, _>(
//!     acquire_effect,
//!     |resource| async move { release(resource).await },
//!     |resource| use_resource(resource),
//! );
//! // Both are guaranteed to have Acquires = Empty, Releases = Empty
//! ```
//!
//! ## Neutrality Assertion
//!
//! Verify resource neutrality at compile time:
//!
//! ```rust,ignore
//! fn process() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
//!     let effect = /* ... */;
//!     assert_resource_neutral(effect)  // Compile error if not neutral
//! }
//! ```
//!
//! # Protocol Enforcement Example
//!
//! ```rust,ignore
//! use stillwater::effect::resource::*;
//!
//! // Transaction protocol
//! fn begin_tx() -> impl ResourceEffect<Acquires = Has<TxRes>> { ... }
//! fn commit(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { ... }
//! fn rollback(tx: Tx) -> impl ResourceEffect<Releases = Has<TxRes>> { ... }
//! fn query(tx: &Tx) -> impl ResourceEffect<Acquires = Empty, Releases = Empty> { ... }
//!
//! // Correct: transaction is opened and closed
//! fn transfer_funds() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
//!     bracket::<TxRes>()
//!         .acquire(begin_tx())
//!         .release(|tx| async move { commit(tx).run(&()).await })
//!         .use_fn(|tx| query(tx).and_then(|_| query(tx)))
//! }
//!
//! // Compile error: transaction never closed
//! fn bad_transfer() -> impl ResourceEffect<Acquires = Empty, Releases = Empty> {
//!     begin_tx()
//!         .and_then(|tx| query(&tx))
//!     // Error: TxRes acquired but not released
//! }
//! ```
//!
//! # Backward Compatibility
//!
//! All existing Effect code continues to work unchanged. Resource tracking
//! is purely additive and opt-in.
//!
//! # Zero-Cost Abstraction
//!
//! The entire resource tracking system is zero-cost:
//! - All marker types are zero-sized
//! - `Tracked` wrapper has the same runtime behavior as the inner effect
//! - Type-level computations happen at compile time only
//! - No runtime checks, allocations, or indirection

pub mod bracket;
pub mod builder;
pub mod combinators;
pub mod ext;
pub mod markers;
pub mod sets;
pub mod tracked;

// Re-export main types
pub use bracket::{resource_bracket, tracked_resource_bracket, ResourceBracket};
pub use builder::{bracket, Bracket, BracketWithAcquire, BracketWithRelease};
pub use ext::{assert_resource_neutral, IsResourceNeutral, ResourceEffectExt, TrackedExt};
pub use markers::{DbRes, FileRes, LockRes, ResourceKind, SocketRes, TxRes};
pub use sets::{Contains, Empty, Has, ResourceSet, Subset, Union};
pub use tracked::{ResourceEffect, Tracked};
